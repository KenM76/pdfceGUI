//! # `shell::commands::reach` — the sixth obligation: a registered command
//! must be **reachable**
//!
//! `HANDOFF.md` §5 lists five obligations that follow from registering a
//! command, and every one of them fails loudly: a count assertion, a group
//! assertion, a `PLANNED` disjointness test, a RON round-trip, a `KNOWN`
//! lookup. **None of the five asks whether the command does anything**, and
//! that is the gap this module closes.
//!
//! ## What went wrong, and how many surfaces agreed with it
//!
//! `file.save_copy` was registered, drawn on the **quick-access toolbar**,
//! bound to `Ctrl+S`, listed in the shortcuts reference, and printed
//! "(Ctrl+S)" in its own tooltip — with **no dispatch arm**. Nothing this
//! shell built could be written to disk, for the whole life of the project,
//! and it was within an hour of being released that way. An audit the same
//! day found the identical shape in `edit.undo`/`edit.redo` (QAT, three
//! chords) and in **every** page operation, six of which the Pages panel's
//! context menu offered while `panels/pages/select.rs` maintained a
//! multi-select model to feed them.
//!
//! Five surfaces promise a command works — the registry, the ribbon, the
//! QAT, the keymap, the tooltip — and **not one of them is a dispatch arm.**
//! The only honest signal is a `command-unimplemented` line in the trace, and
//! nothing read it.
//!
//! ## The property asserted here
//!
//! > Every command in the registry is either named by a **literal arm** of
//! > `PdfceApp::dispatch_command`'s `match`, or claimed by one of its
//! > **guard arms**, or listed in [`SCAFFOLDED`] with a written reason.
//!
//! It is a statement about **routing**, not about behaviour. An arm that
//! declines — `measure.finish` refusing because the mode cannot author
//! dimensions — is reachable, and correctly so: the operator's press produced
//! a decision instead of falling through to `command-unimplemented`. What this
//! catches is the *absence of a decision*.
//!
//! # ★ Why the arms are READ from the source rather than run
//!
//! Three mechanisms were available. The two that lost are worth recording,
//! because each lost for a reason a future session would otherwise re-derive.
//!
//! ## Rejected — dispatch every command in a test and assert it was handled
//!
//! The truest signal, and unavailable. `crate::app::files`' header states the
//! rule in its own words as **Rule 3: no test may dispatch `file.open`** —
//! "on the machine this is built on, dispatching `file.open` opens a **real
//! modal dialog** and blocks until a human dismisses it. A `cargo test` that
//! did that would hang the suite with an invisible window behind the
//! terminal."
//!
//! The escape hatch does not open either. `PDFCE_DIAG_OPEN_PATH` answers the
//! dialog without a human, but setting it needs `std::env::set_var`, which is
//! `unsafe` in edition 2024 while this crate is `#![forbid(unsafe_code)]` —
//! the same wall that leaves `files::from_env`'s environment read as its one
//! untested millimetre.
//!
//! So a dispatching test would have to **exempt `file.open`**, and that is
//! decisive rather than inconvenient: `file.open` is the command this defect
//! struck with the largest blast radius — registered, on the File tab, on the
//! QAT, bound to `Ctrl+O`, and armless, so the only way to open a document was
//! `argv`. A check that must skip the worst historical instance of the defect
//! it exists to prevent is not a check.
//!
//! (The other hazards are real but were *not* the deciding ones, and it is
//! worth saying which, so nobody re-opens this on the wrong grounds. Almost
//! every arm is safe in a test, because of the invariant `HANDOFF.md` §6 puts
//! first: **actions, not mutations.** `pages.delete` pushes
//! `Action::DeletePages` into a `Vec` the caller owns and deletes nothing;
//! `file.ocr` sets a dialog-open flag and starts no recogniser. The genuinely
//! effectful arms are the clipboard writes and the native picker, and of those
//! only the picker cannot be reached in a state where it does nothing.)
//!
//! ## Rejected — a `bash` gate that greps `dispatch.rs`
//!
//! Honest for a gate, and wrong for this file, for two independent reasons.
//!
//! **A `match` is not a regular language.** The failure that matters is a
//! *false pass*, and a grep for `"some.id" =>` has at least three ways to
//! produce one: a string inside a comment, the left-hand side of a **nested**
//! `match` inside an arm's body (`dispatch_command` contains four), and a doc
//! comment quoting an id whose arm has since been deleted — which is the D5
//! shape exactly, a list that agrees with itself about something that stopped
//! being true. `check-ui-strings.sh`'s header records the same class of error
//! from the other side: its first regex read `"svg" | "?xml"` as one literal
//! containing `" | "`, and "three of the four remaining hits… were exactly
//! that artefact — i.e. most of what was left after the real exclusions was
//! the detector misreading Rust, not the code violating the rule."
//!
//! **The guard arms are expressions, and a shell cannot evaluate them.** Six
//! arms have the shape `id if …_for_command(id).is_some()`, and the functions
//! behind them search enum tables in three other modules. A gate that decided
//! which ids they claim would have to re-derive `markup_command`,
//! `measure_command`, `chrome_command`, `page_display_command`,
//! `text_mark_command` and `Panel::command_id` from source — six more `match`
//! blocks to parse, and, worse, a **second table** of exactly the kind
//! [`super::mapping`]'s header exists to forbid: *"two hand-written tables can
//! disagree, and one table plus a derived search cannot."*
//!
//! ## Rejected — restructure so one `fn arm_for(id) -> Option<Arm>` is the
//! source of truth
//!
//! The best signal and the most invasive, and it contradicts the file it would
//! restructure. `app::dispatch`'s header states the property being protected:
//! **"the arms route; they do not compute"**, each arm one line that pushes an
//! `Action` or calls the one function that owns the rule. Interposing a table
//! turns every arm into two lookups — a variant, then a body — and the thing
//! a reader currently gets for free, that `"file.close" => actions.push(
//! Action::Close)` is the whole story, is precisely what would be lost. The
//! brief for this work says so in the same words: it must not turn one
//! readable `match` into a table nobody can follow.
//!
//! # ★ What is done instead, and why it is not the rejected grep
//!
//! **The literal arms are read from the abstract syntax tree**, with `syn` —
//! a real Rust parser, already in `D:\Dev\pdfce`'s lockfile, taken as a
//! dev-dependency on the standard `flate2` and `rfd` are held to (see this
//! crate's `Cargo.toml`). Every objection above is an objection to treating
//! Rust as text, and none of them survives parsing it as Rust: an arm pattern
//! is an `Arm`'s `pat`, a comment is not in the tree at all, and a nested
//! `match` inside an arm's **body** is never visited, because only the arms of
//! the one `match` on `id` are read. `the_reader_does_not_see_a_nested_match`
//! pins that last one against a fixture, since it is the case a grep gets
//! wrong and the case nobody would notice.
//!
//! **The guard arms are not parsed. They are CALLED.** The tree says *which*
//! functions guard arms consult ([`Arms::guards`], the last path segment of
//! whatever is invoked with `id`); Rust then answers, for every registered id,
//! whether any of them claims it — by running the real
//! [`super::markup_for_command`] and its five siblings against the real
//! registry. There is no second table, because there is no table: the one
//! `match` in [`super::mapping`] is the only statement of each mapping, and
//! this reads it by executing it.
//!
//! The two halves are then held together by
//! [`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
//! which asserts that the guard names found **in the source** and the guard
//! names this module **evaluates** are the same set. That closes the hole a
//! hand-kept list would otherwise open in both directions: a *seventh* guard
//! arm added to `dispatch.rs` fails by name rather than silently reporting its
//! whole family unreachable, and a guard arm *deleted* from `dispatch.rs`
//! stops making its family reachable rather than being vouched for by a
//! function that still exists.
//!
//! # Why this is a test and not a `tools/gates/` script
//!
//! Because both halves of the answer are Rust. The registry is built by
//! [`super::register`], the guards are functions with enum tables behind them,
//! and a shell script could reach neither without re-deriving both — which is
//! the failure the paragraphs above are about. What a gate script contributes
//! that a test does not is a *precondition* guarantee, and this has a stronger
//! one than any script can offer: [`DISPATCH_SRC`] is an `include_str!`, so a
//! dispatcher that has been moved, renamed or deleted **fails to compile**
//! rather than being quietly scanned as an empty tree. `run-all.sh`'s
//! three-state model exists because "found nothing" and "looked at nothing"
//! print the same thing; here the second state cannot be reached.
//!
//! # What it found, and what has been closed since
//!
//! On the day this landed: **38 of the 101 registered commands had no dispatch
//! arm.** Every one was drawn, enabled by its predicate and pressable, and every
//! one traced `command-unimplemented`. All 38 were listed in [`SCAFFOLDED`] with
//! the reason each was inert, and **11 carried a `★ P3` mark** — this module's
//! judgement that the honest answer is *the control should not be drawn yet*
//! (`RIBBON_IA.md` P3: "An unavailable capability renders nothing, not a
//! disabled stub").
//!
//! **It is now 35 and 8.** Three of the eleven were wired the next day rather
//! than argued for: `view.read_mode` and `view.fullscreen` (`app::window` — and
//! `view.read_mode` was first established *not* to be a duplicate of
//! `mode.read`, which would have made deletion the honest answer instead), and
//! `tools.render_diagnostics` (`dialogs::diagnostics`, the readout moved off the
//! status bar to a surface with room for it). A fourth, `view.show_points`, was
//! investigated and **deliberately left here**: the reason it had none was that
//! there is nothing for it to show — see its entry — which is a better outcome
//! than a toggle that toggles nothing.
//!
//! Both figures are pinned by [`tests::the_p3_tension_is_counted`], for the
//! reason `the_icon_coverage_split_adds_up_to_the_registry` exists: a count
//! quoted in prose and pinned by nothing has drifted in this crate four times.
//!
//! Whether a `★ P3` entry loses its control is a **taxonomy decision and the
//! operator's**; nothing here removes one. What this module can do is make the
//! number impossible to lose track of, and make it go *down* rather than
//! sideways — which is what it has now done once.
//!
//! ★ **And it found the mirror defect, which nobody was looking for.** Four
//! literal arms — `view.zoom_in`, `view.zoom_out`, `view.next_page` and
//! `view.prev_page` — named commands that are **not registered at all**, so no
//! token could reach them and no operator ever had. All four were **deleted** on
//! 2026-08-15, after each verb was checked to have two live routes that are not
//! the dispatcher. [`UNREACHED_ARMS`] is therefore empty and is kept as a gate:
//! it exists because the first planted violation of this check was one of those
//! arms and the check said nothing.
//!
//! The gate discipline is kept in full. [`tests`] contains a self-test that
//! plants a violation in a fixture and proves the reader reports it, another
//! that proves the reader does **not** report a clean fixture, and two that
//! aim at the specific misreadings a grep would make — because, in
//! `check-file-size.sh`'s words, a gate that has never been observed to fail
//! is not evidence of anything.

use std::collections::{BTreeMap, BTreeSet};

/// The dispatcher's own source, embedded at **compile** time.
///
/// `include_str!` rather than a runtime `std::fs::read_to_string` of a path
/// built from `CARGO_MANIFEST_DIR`, and the difference is the whole
/// precondition story. A path that stops resolving is a *runtime* `Err` that
/// somebody has to remember to treat as a failure; `run-all.sh`'s header is
/// about exactly that ("SKIPPED is not PASSED"). A missing `include_str!`
/// target is a **compile error**, so the state in which this module checks
/// nothing and says so quietly does not exist.
const DISPATCH_SRC: &str = include_str!("../../app/dispatch.rs");

/// This module's parent, read for the `&'static str` constants that arm
/// patterns may name.
///
/// One arm of the dispatcher is written `crate::shell::commands::FILE_RECENT
/// => …` rather than as a literal, for the reason that constant's own doc
/// comment gives: the id is spelled in four places that must agree, and "a
/// typo in any of them produces silence — a menu that draws and reports
/// nothing — rather than an error."
///
/// So the reader resolves constant patterns instead of ignoring them, and it
/// resolves them **by parsing the file that defines them** rather than by
/// carrying a copy of the value. A copy would be a hand-maintained mirror of
/// exactly one entry, which is still the thing `DEFECTS.md` D5 forbids: *"a
/// hand-maintained list with a comment telling you to hand-maintain it has
/// already failed once."*
const CONSTS_SRC: &str = include_str!("mod.rs");

/// The method whose `match` is the routing table.
// ui-text-exempt: a Rust item name, matched against the parsed syntax tree.
const DISPATCHER: &str = "dispatch_command";

/// The identifier the routing `match` scrutinises, and that its guard arms
/// pass to the mapping functions.
// ui-text-exempt: a Rust binding name, matched against the parsed syntax tree.
const SUBJECT: &str = "id";

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
pub(super) const SCAFFOLDED: &[(&str, &str)] = &[
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
        "file.settings",
        "Blocked on the settings dialog, which is salvaged-but-unbuilt. `FEATURES.md`: \
         “Settings dialog — the spec-ambiguity model, plus the new Render group” (⬜), and \
         the theme row states the live consequence in its own words: “The preset is not yet \
         choosable — that is the settings dialog, still unsalvaged.” An arm here would open \
         a window this build does not have.",
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
    (
        "view.render_strategy",
        "`manifest::DIRECTED`: “Whole page · Tiled progressive. Named explicitly when the \
         View ▸ Render group was commissioned; whole-page is the default because it measured \
         better in use.” Drawn as a button rather than as the two-position control that \
         argument assumes — see this group's note above.",
    ),
    (
        "view.render_quality",
        "`manifest::DIRECTED`: “partial G — the raster-scale multiplier is a compiled-in \
         constant today. What is new is the knob, not the value.” The knob is still the \
         part that is missing; a press does nothing. See this group's note above.",
    ),
    (
        "view.render_settle",
        "`manifest::DIRECTED`: “partial G — `ZOOM_SETTLE` is a compiled-in constant today.” \
         Its twin one row up carries the same status for the raster scale. Drawn as a button \
         rather than as a value control — see this group's note above.",
    ),
    (
        "view.render_thin_lines",
        "`manifest::DIRECTED`: “Named explicitly when the View ▸ Render group was \
         commissioned. RIBBON_IA.md §5.2 lists it under both Render and Display; it is kept \
         here, once.” See this group's note above for the gap between that argument and \
         what is drawn.",
    ),
    (
        "view.render_antialias",
        "`manifest::DIRECTED`: “Named explicitly when the View ▸ Render group was \
         commissioned (text / vector).” The value set the specification gave it has no \
         control to show it in yet — see this group's note above.",
    ),
    (
        "view.floating_panels",
        "`manifest::DIRECTED`: “Off · Allowed, default Allowed. Operator decision \
         2026-08-13, retiring ‘nothing floats over the canvas’ as an absolute. Off \
         restores today's behaviour exactly.” The default is the shipped behaviour, so the \
         inert control changes nothing an operator can observe — which is the strongest \
         version of that register's argument and still not the same as showing a value.",
    ),
    (
        "view.app_initiative",
        "`manifest::DIRECTED`: “Never · Ask · Allowed, default NEVER. The half that carries \
         the original complaint; its default preserves the shipped behaviour as a choice \
         rather than a law.” As its neighbour above: the default is what ships, so nothing \
         is misreported — but nothing is shown either.",
    ),
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
        "★ P3 — as its sibling above, and blocked harder. `app::dispatch`'s table: it needs \
         “a file picker plus an insertion point, and — the blocking half — `insert` returns \
         the bytes of a NEW document rather than mutating the session. Wiring it means \
         replacing `OpenDoc::session` wholesale, which discards the command log the undo \
         work is building.” An architectural decision, not a wiring job.",
    ),
    (
        "pages.insert_from_file",
        "★ P3 — the twin of the entry above, blocked on the same two things. `app::dispatch`'s \
         table: “the manifest's own table separates them by WHERE the pages land, not by \
         which engine verb runs.” Neither is in the Pages context menu, which the panel's \
         own test records as a deliberate exclusion — so the breach is the ribbon tab alone.",
    ),
    // ===================================================================
    // EDIT
    // ===================================================================
    (
        "edit.text",
        "Phase 5, deferred by the operator, and the deferral is checked rather than assumed: \
         `app::modes::capability` records that the chord/mode gate it sits behind was \
         “latent rather than live”, because “every chord-bound Edit command reaches \
         `command-unimplemented` at the time of writing. Phase 5 is what makes it live.” \
         `FEATURES.md`'s backlog row is “Text editing — the whole tool”.",
    ),
    (
        "edit.add_text",
        "The other half of the same deferred phase — placing new text rather than editing \
         what is there. `app::modes::capability` lists both among the chord-bound Edit \
         commands that reach `command-unimplemented`, and `FEATURES.md` carries one backlog \
         row for the tool rather than two, because the two verbs share it.",
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
    (
        "edit.redact",
        "Salvage. `FEATURES.md`: “Redaction — mark, review, apply, with the true-removal \
         proof that exists only in the old shell.” `SALVAGE.md`'s row for `redact_apply.rs` \
         is stronger still: “★ This file is currently the ONLY place the proof exists.” \
         Marking without the proof would be the worse half to ship first.",
    ),
    (
        "edit.redact_apply",
        "The other half of the one unsalvaged unit above — `FEATURES.md` names “mark, review, \
         apply” as a single row for that reason. This is the verb the true-removal proof \
         belongs to, and the proof is the part that has not crossed over.",
    ),
    // ===================================================================
    // MARKUP
    // ===================================================================
    (
        "markup.text_box",
        "A different gesture and a different spec type. `canvas::markup`'s table of kinds it \
         deliberately does not carry: “Note · text box · sticky · stamp — Text-bearing, not \
         geometric. A different gesture (place, then type) and a different spec type \
         (`TextAnnotSpec`).” Nothing about the drag-and-release machinery the seven shipped \
         kinds share applies to it.",
    ),
    (
        "markup.sticky_note",
        "Named in the same row of `canvas::markup`'s table as the entry above, for the same \
         two reasons: it is text-bearing rather than geometric, so it needs a place-then-type \
         gesture and `TextAnnotSpec` rather than `MarkupSpec`.",
    ),
    (
        "markup.stamp",
        "The third kind in that same row, plus one blocker of its own: `manifest/markup.rs` \
         records that “the stamp control exists and needs a GALLERY, which is a change to \
         the control rather than a new command”. A stamp with no chooser has no operand.",
    ),
    // ===================================================================
    // MEASURE
    // ===================================================================
    (
        "measure.set_scale",
        "The clearest statement of a missing arm in the crate, and it is already written \
         down. `canvas::rulers`: “**The limit, stated rather than left to be discovered:** \
         this build has no way to SET a scale. It is registered and has no dispatch arm, and \
         `EditSession`'s scale verbs have no caller in the GUI.” `FEATURES.md` agrees the \
         scale model “waits on a dialog rather than on a decision”.",
    ),
    (
        "measure.manage_groups",
        "Waits on the same absent dialog. `super::mapping`'s test records what it must NOT \
         become in the meantime: “deliberately NOT a tool: it opens a dialog. If it ever \
         answered here it would arm a picking state the operator never asked for.” A list \
         you add to, rename in and remove from is a window, not an arm.",
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
const UNREACHED_ARMS: &[(&str, &str)] = &[];

// ===========================================================================
// READING THE ARMS
// ===========================================================================

/// What one `match` offers: the ids its literal arms name, and the guard
/// functions its guard arms consult.
///
/// Both are sets rather than lists because the question asked of them is only
/// ever membership, and because a duplicate arm is a `match` the compiler
/// already warns about.
#[derive(Debug, Default)]
pub(super) struct Arms {
    /// Every id named by an arm pattern — a string literal, an alternation of
    /// them, or a path naming a `&'static str` constant that resolves.
    pub(super) literals: BTreeSet<String>,
    /// The **last path segment** of each function a guard arm calls with the
    /// subject: `markup_for_command`, `from_command_id`, and so on.
    ///
    /// The last segment rather than the whole path, because the path is a
    /// spelling decision (`crate::shell::commands::markup_for_command` here,
    /// a `use` away from `markup_for_command` in a future edit) and the
    /// function is the fact.
    pub(super) guards: BTreeSet<String>,
    /// Whether a catch-all arm — a binding or `_` — is present.
    ///
    /// Asserted rather than used: the catch-all is where
    /// `command-unimplemented` is traced, so a `match` without one is not the
    /// `match` this module thinks it is reading.
    pub(super) catch_all: bool,
}

/// Read the routing table out of `src`.
///
/// `src` is a parameter rather than a reach for [`DISPATCH_SRC`] for the
/// reason `crate::diag::record_if_changed` takes its map as an argument: the
/// **rule** is the interesting part and it has to be testable against a
/// fixture. A reader that can only be pointed at the real file cannot be shown
/// to bite.
///
/// # Errors
///
/// Returns the reason as a string when the source does not parse, when no
/// method named [`DISPATCHER`] holds a `match`, or when an arm pattern is a
/// shape this reader does not classify. **All three fail closed**: an
/// unreadable dispatcher reports *nothing* reachable rather than everything,
/// which is the direction that makes a caller notice.
pub(super) fn read_arms(src: &str, consts: &BTreeMap<String, String>) -> Result<Arms, String> {
    let file = syn::parse_file(src).map_err(|e| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("the dispatcher does not parse as Rust: {e}")
    })?;
    let matched = find_routing_match(&file).ok_or_else(|| {
        // ui-text-exempt: a test failure message, never displayed to an operator.
        format!("no `match` was found in a method named `{DISPATCHER}`")
    })?;

    let mut arms = Arms::default();
    for (n, arm) in matched.arms.iter().enumerate() {
        // A guard arm is classified by what it CALLS, never by what it
        // matches: its pattern is the binding `id`, which names no command.
        if let Some((_, guard)) = &arm.guard {
            let name = guard_subject_fn(guard).ok_or_else(|| {
                // ui-text-exempt: a test failure message, never displayed to an operator.
                format!(
                    "arm {n} is guarded by an expression that calls nothing with `{SUBJECT}`; \
                     this reader cannot tell which commands it claims"
                )
            })?;
            arms.guards.insert(name);
            continue;
        }
        collect_pattern(&arm.pat, consts, n, &mut arms)?;
    }
    Ok(arms)
}

/// Classify one arm pattern into [`Arms`].
///
/// Split from [`read_arms`] because `Pat::Or` recurses into it once per
/// alternative — `"pages.rotate_left" | "pages.rotate_right"` is one arm and
/// two ids — and writing that inline would put the classification rule in two
/// places.
fn collect_pattern(
    pat: &syn::Pat,
    consts: &BTreeMap<String, String>,
    n: usize,
    arms: &mut Arms,
) -> Result<(), String> {
    match pat {
        // `"file.new" => …`
        syn::Pat::Lit(lit) => match &lit.lit {
            syn::Lit::Str(s) => {
                arms.literals.insert(s.value());
                Ok(())
            }
            // ui-text-exempt: a test failure message, never displayed to an operator.
            _ => Err(unclassifiable(n, "a non-string literal pattern")),
        },
        // `"a" | "b" => …`
        syn::Pat::Or(or) => or
            .cases
            .iter()
            .try_for_each(|case| collect_pattern(case, consts, n, arms)),
        // `crate::shell::commands::FILE_RECENT => …`
        //
        // Resolved through the constant table, and **unresolvable is an
        // error rather than a shrug**: a path pattern this reader cannot
        // resolve is an arm whose id it does not know, and silently
        // dropping it is how a real arm comes to look like no arm at all.
        syn::Pat::Path(path) => {
            let last = path
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            match consts.get(&last) {
                Some(value) => {
                    arms.literals.insert(value.clone());
                    Ok(())
                }
                None => Err(format!(
                    // ui-text-exempt: a test failure message, never displayed to an operator.
                    "arm {n} matches the path `…::{last}`, which is not a `&str` constant \
                     this reader can resolve"
                )),
            }
        }
        // `other => …` / `_ => …`
        syn::Pat::Ident(_) | syn::Pat::Wild(_) => {
            arms.catch_all = true;
            Ok(())
        }
        _ => Err(unclassifiable(
            n,
            // ui-text-exempt: a test failure message, never displayed to an operator.
            "a pattern shape this reader does not know",
        )),
    }
}

/// The message for an arm this reader will not guess at.
///
/// Failing rather than ignoring is the whole discipline: an arm the reader
/// cannot classify is an arm whose ids it would otherwise report as
/// unreachable *or* miss entirely, and neither silence is acceptable in a
/// check whose only interesting failure is a false pass.
fn unclassifiable(n: usize, what: &str) -> String {
    // ui-text-exempt: a test failure message, never displayed to an operator.
    format!("arm {n} is {what}; teach `collect_pattern` about it rather than ignoring it")
}

/// The `match` that routes commands: the first one found directly in the body
/// of a method named [`DISPATCHER`].
///
/// Deliberately **not** a search for any `match` anywhere in the file.
/// `dispatch_command` contains four nested `match` expressions inside arm
/// bodies (the recent-file operand, the text-mark outcome, the page-move
/// refusal, the page-text failure), and one of them — the refusal — has string
/// literals on the *right* of its arrows. Reading the wrong one is the mistake
/// a grep makes; reading the right one is the reason this walks a tree.
fn find_routing_match(file: &syn::File) -> Option<&syn::ExprMatch> {
    file.items.iter().find_map(|item| {
        let syn::Item::Impl(imp) = item else {
            return None;
        };
        imp.items.iter().find_map(|member| {
            let syn::ImplItem::Fn(f) = member else {
                return None;
            };
            if f.sig.ident != DISPATCHER {
                return None;
            }
            f.block.stmts.iter().find_map(|stmt| match stmt {
                syn::Stmt::Expr(syn::Expr::Match(m), _) => Some(m),
                _ => None,
            })
        })
    })
}

/// The name of the function a guard arm calls with the subject binding.
///
/// Every guard arm in the dispatcher has the shape
/// `id if <path>(id).is_some()`, so this peels the wrappers — a method call, a
/// negation, a parenthesis — until it finds a call whose single argument is
/// the subject, and returns that call's last path segment.
///
/// Requiring the argument to be **exactly the subject** is what stops it
/// answering for an unrelated call in a more complicated guard. A guard that
/// does something this cannot read is an error at the call site above, not a
/// silent `None` treated as "claims nothing".
fn guard_subject_fn(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::MethodCall(m) => guard_subject_fn(&m.receiver),
        syn::Expr::Unary(u) => guard_subject_fn(&u.expr),
        syn::Expr::Paren(p) => guard_subject_fn(&p.expr),
        syn::Expr::Binary(b) => guard_subject_fn(&b.left).or_else(|| guard_subject_fn(&b.right)),
        syn::Expr::Call(call) => {
            let syn::Expr::Path(func) = &*call.func else {
                return None;
            };
            if call.args.len() != 1 {
                return None;
            }
            let Some(syn::Expr::Path(arg)) = call.args.first() else {
                return None;
            };
            if !arg.path.is_ident(SUBJECT) {
                return None;
            }
            func.path.segments.last().map(|s| s.ident.to_string())
        }
        _ => None,
    }
}

/// Every `&'static str` constant declared at the top level of `src`.
///
/// Only `const NAME: &str = "value";` is recognised, which is the one shape an
/// arm pattern can name. A constant built from an expression is not a pattern
/// Rust would accept either, so nothing is lost by not resolving one.
pub(super) fn string_consts(src: &str) -> BTreeMap<String, String> {
    let Ok(file) = syn::parse_file(src) else {
        return BTreeMap::new();
    };
    file.items
        .iter()
        .filter_map(|item| {
            let syn::Item::Const(c) = item else {
                return None;
            };
            let syn::Expr::Lit(lit) = &*c.expr else {
                return None;
            };
            let syn::Lit::Str(s) = &lit.lit else {
                return None;
            };
            Some((c.ident.to_string(), s.value()))
        })
        .collect()
}

// ===========================================================================
// ASKING THE GUARDS, BY RUNNING THEM
// ===========================================================================

/// The guard function that claims `id`, if any — **by calling it**.
///
/// This is the half a shell script could not have written. Each name returned
/// is the same string [`read_arms`] extracts from the guard arm that consults
/// it, so the two halves can be compared as sets; and each answer comes from
/// the real mapping rather than from a re-derivation of it, so there is no
/// second table to drift. [`super::mapping`]'s header states the property this
/// preserves: *"two hand-written tables can disagree, and one table plus a
/// derived search cannot."*
///
/// Order is irrelevant here even though it is load-bearing in the dispatcher,
/// where `match` takes the first arm that matches. Reachability asks only
/// whether **some** arm claims the id; which one wins is asserted, in both
/// directions, by the disjointness tests in [`super::mapping`].
pub(super) fn guard_claiming(id: &str) -> Option<&'static str> {
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    if super::measure_for_command(id).is_some() {
        return Some("measure_for_command");
    }
    if super::text_mark_for_command(id).is_some() {
        return Some("text_mark_for_command");
    }
    if super::markup_for_command(id).is_some() {
        return Some("markup_for_command");
    }
    if super::page_display_for_command(id).is_some() {
        return Some("page_display_for_command");
    }
    if super::chrome_for_command(id).is_some() {
        return Some("chrome_for_command");
    }
    if crate::panels::Panel::from_command_id(id).is_some() {
        return Some("from_command_id");
    }
    None
}

/// Every guard [`guard_claiming`] knows how to run.
///
/// **Not a mirror of the dispatcher**, and the distinction is the one `D5`
/// turns on: this list is *asserted equal* to the set read out of
/// `dispatch.rs`'s syntax tree by
/// [`tests::the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has`],
/// so it cannot drift without a named failure. A hand-maintained list that
/// nothing checks is the defect; a hand-written list that a test pins against
/// the source is a declaration.
const EVALUATED_GUARDS: &[&str] = &[
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "measure_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "text_mark_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "markup_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "page_display_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "chrome_for_command",
    // ui-text-exempt: Rust function names, compared against the parsed syntax tree.
    "from_command_id",
];

/// Whether `id` is routed by some arm of `arms`.
///
/// ★ The guard half consults **both** sides: a guard function may claim the
/// id, *and* the dispatcher must actually have an arm that consults that
/// function. Checking only the first would keep vouching for a family whose
/// guard arm had been deleted — the mapping would still answer and four ribbon
/// buttons would silently stop working, which is precisely the shape
/// [`super::mapping`]'s header warns about one level down.
pub(super) fn is_routed(id: &str, arms: &Arms) -> bool {
    arms.literals.contains(id)
        || guard_claiming(id).is_some_and(|guard| arms.guards.contains(guard))
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_shell::CommandRegistry;

    /// The live registry, built the way `PdfceApp` builds it.
    ///
    /// Against **this** rather than against [`super::super::catalog::all`]'s
    /// literal text, for the reason every test in [`super::super::mapping`]
    /// gives: it is the difference between asserting that the code agrees with
    /// itself and asserting that the control exists.
    fn registry() -> CommandRegistry {
        let mut reg = CommandRegistry::new();
        super::super::register(&mut reg);
        reg
    }

    /// The real dispatcher's arms, or a panic naming why they could not be
    /// read.
    fn dispatcher() -> Arms {
        read_arms(DISPATCH_SRC, &string_consts(CONSTS_SRC))
            .expect("the dispatcher must be readable")
    }

    /// Every registered id the dispatcher does not route.
    fn unrouted() -> Vec<String> {
        let arms = dispatcher();
        registry()
            .iter()
            .map(|c| c.id.clone())
            .filter(|id| !is_routed(id, &arms))
            .collect()
    }

    // -----------------------------------------------------------------
    // THE CHECK
    // -----------------------------------------------------------------

    /// ★★ **Every registered command is reachable, or argued for.**
    ///
    /// The one assertion this module exists to make. A failure here means a
    /// control is drawn, enabled and pressable and produces
    /// `command-unimplemented` — which is what `file.save_copy` did for the
    /// whole life of the project, agreed with by five surfaces and contradicted
    /// by none.
    #[test]
    fn every_registered_command_is_routed_or_argued() {
        let argued: BTreeSet<&str> = SCAFFOLDED.iter().map(|(id, _)| *id).collect();
        let orphans: Vec<String> = unrouted()
            .into_iter()
            .filter(|id| !argued.contains(id.as_str()))
            .collect();
        assert!(
            orphans.is_empty(),
            "{} registered command(s) have no dispatch arm and no argued exemption: {}\n\
             \n\
             Each one is a control an operator can press that traces \
             `command-unimplemented` and does nothing. Write the arm in \
             `app/dispatch.rs`, or add the id to `SCAFFOLDED` with the REASON it \
             is deliberately inert — and if the honest reason is that it should \
             not be drawn yet, say so there rather than here.",
            orphans.len(),
            orphans.join(", ")
        );
    }

    /// **No entry on the allow-list has rotted.**
    ///
    /// Three ways an exemption goes stale, and all three are silent:
    ///
    /// * the command is **no longer registered** — the entry then excuses an id
    ///   nothing has, and reads as a live promise that the control exists;
    /// * the command **has been wired** — the entry then states a reason that
    ///   is false, and the next reader believes it;
    /// * the reason has decayed into a restatement of the id, which is the
    ///   thing the brief for this list specifically forbids.
    ///
    /// The middle one is the important one. Without it this list is a place to
    /// park a command permanently, and an allow-list nobody ever has to shorten
    /// is `DEFECTS.md` D5's *"hand-maintained list with a comment telling you to
    /// hand-maintain it"* wearing a different hat.
    #[test]
    fn no_scaffolded_entry_is_stale() {
        let reg = registry();
        let arms = dispatcher();
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (id, reason) in SCAFFOLDED {
            assert!(
                seen.insert(id),
                "`{id}` is on the allow-list twice; one command, one reason"
            );
            assert!(
                reg.get(id).is_some(),
                "`{id}` is on the allow-list and is not registered. \
                 An exemption for a command that does not exist excuses nothing \
                 and misleads the next reader; delete it, or, if the command was \
                 renamed, follow it."
            );
            assert!(
                !is_routed(id, &arms),
                "`{id}` is on the allow-list AND has a dispatch arm. \
                 The entry now states a reason that is false — the work landed. \
                 Delete the entry."
            );
            assert!(
                reason.len() >= 40 && !reason.contains(id),
                "`{id}`'s allow-list entry must carry the REASON, not the name. \
                 Cite the place the reason already lives — a registration's doc \
                 comment, `app::dispatch`'s own table, a `SALVAGE.md` class, or a \
                 `FEATURES.md` row — rather than writing a second wording that can \
                 drift from the first."
            );
        }
    }

    /// ★ **No literal arm names a command that is not registered.**
    ///
    /// The mirror of the check above, and it was not planned — it fell out of
    /// planting the first violation. `app::dispatch`'s `format.delete` arm
    /// states the rule it enforces: an arm for an unregistered id is *"an arm
    /// no token can ever reach — dead code wearing a design pattern, which is
    /// what the no-placeholders invariant forbids"*.
    ///
    /// The failure is quieter than the one this module was written for, and in
    /// one way nastier: an inert control at least *looks* wrong when pressed,
    /// while a dead arm reads as working code and will be maintained, reviewed
    /// and reasoned about by everyone who passes it.
    ///
    /// Guard arms are deliberately not checked here. They claim ids by
    /// computing over an enum, and [`super::mapping`]'s own tests already
    /// assert in both directions that every kind has a registered command.
    #[test]
    fn no_literal_arm_names_an_unregistered_command() {
        let reg = registry();
        let tolerated: BTreeSet<&str> = UNREACHED_ARMS.iter().map(|(id, _)| *id).collect();
        let arms = dispatcher();
        let dead: Vec<&String> = arms
            .literals
            .iter()
            .filter(|id| reg.get(id).is_none() && !tolerated.contains(id.as_str()))
            .collect();
        assert!(
            dead.is_empty(),
            "{} dispatch arm(s) name a command that is not registered, so no token can \
             ever reach them: {dead:?}\n\
             \n\
             Delete the arm, or register the command it is waiting for — and if it is \
             deliberate, put it in `UNREACHED_ARMS` with the reason.",
            dead.len()
        );
        // …and the tolerated list itself must not rot: an entry that HAS been
        // registered since is an arm that now works, and the note excusing it
        // has become false.
        for (id, reason) in UNREACHED_ARMS {
            assert!(
                reg.get(id).is_none(),
                "`{id}` is listed as unreachable and is now registered; delete the entry"
            );
            assert!(reason.len() >= 40 && !reason.contains(id));
        }
    }

    /// **The allow-list and `PLANNED` describe different states and must not
    /// overlap.**
    ///
    /// `manifest::PLANNED` is for commands that are **not registered**: named
    /// by `RIBBON_IA.md`, absent from this build, drawn nowhere. Everything
    /// here is registered and drawn. An id in both lists would mean one of the
    /// two is wrong about whether the command exists, and the registry
    /// assertion in [`no_scaffolded_entry_is_stale`] already says which.
    #[test]
    fn no_scaffolded_command_is_also_planned() {
        let planned: BTreeSet<&str> = crate::shell::manifest::PLANNED
            .iter()
            .map(|(id, _)| *id)
            .collect();
        for (id, _) in SCAFFOLDED {
            assert!(
                !planned.contains(id),
                "`{id}` is both PLANNED (not registered) and SCAFFOLDED (registered, \
                 no arm). Those are different states and it cannot be in both."
            );
        }
    }

    /// ★ **The guards the checker runs are the guards the dispatcher has.**
    ///
    /// The seam between the two halves of this module, asserted as a set
    /// equality in both directions:
    ///
    /// * a **new** guard arm in `dispatch.rs` that [`guard_claiming`] cannot
    ///   run would otherwise report that arm's whole family unreachable, and
    ///   the reader would go looking in the wrong place;
    /// * a **deleted** guard arm would otherwise keep being vouched for by a
    ///   mapping function that still exists, which is a false pass on the
    ///   exact defect this module is about.
    #[test]
    fn the_guards_the_checker_evaluates_are_the_guards_the_dispatcher_has() {
        let in_source = dispatcher().guards;
        let evaluated: BTreeSet<String> =
            EVALUATED_GUARDS.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(
            in_source, evaluated,
            "`dispatch_command`'s guard arms and `guard_claiming` have diverged. \
             Add the missing function to `guard_claiming` and to `EVALUATED_GUARDS`, \
             or remove the one the dispatcher no longer consults."
        );
    }

    /// The dispatcher still has the catch-all that traces
    /// `command-unimplemented`.
    ///
    /// A sanity check on the **reading**, not on the code: a `match id` with no
    /// catch-all would not compile, so a run in which this is false means the
    /// reader found something other than the routing table and every other
    /// assertion in this module is measuring the wrong thing.
    #[test]
    fn the_reader_found_the_routing_table() {
        let arms = dispatcher();
        assert!(
            arms.catch_all,
            "no catch-all arm: this is not the routing table"
        );
        assert!(
            arms.literals.len() > 20,
            "only {} literal arm(s) were read; the reader has lost the routing table",
            arms.literals.len()
        );
    }

    // -----------------------------------------------------------------
    // THE SELF-TEST — the reader proves it bites
    // -----------------------------------------------------------------
    //
    // `check-file-size.sh`'s header states the rule these four keep: a gate
    // that has never been observed to fail is not evidence. Each fixture below
    // is a miniature dispatcher, and between them they plant every misreading
    // that would turn this check green while the defect shipped.

    /// A fixture dispatcher carrying one of each arm shape, plus every trap a
    /// text scan falls into.
    const CLEAN_FIXTURE: &str = r####"
impl PdfceApp {
    pub(super) fn dispatch_command(&mut self, id: &str) {
        // "fx.in_a_comment" must not be read as an arm.
        match id {
            "fx.literal" => self.one(),
            "fx.left" | "fx.right" => self.pair(),
            crate::shell::commands::FX_CONST => self.constant(),
            id if crate::shell::commands::fx_for_command(id).is_some() => self.guarded(),
            other => {
                let _ = "fx.in_a_body";
                match other {
                    "fx.in_a_nested_match" => self.nested(),
                    _ => self.unimplemented(other),
                }
            }
        }
    }
}
"####;

    /// The constant table the fixture's path pattern resolves through.
    const FIXTURE_CONSTS: &str = r####"
pub const FX_CONST: &str = "fx.constant";
"####;

    fn fixture_arms(src: &str) -> Arms {
        read_arms(src, &string_consts(FIXTURE_CONSTS)).expect("the fixture must be readable")
    }

    /// **A. The reader finds every arm shape the dispatcher actually uses.**
    ///
    /// Without this, assertion B below could pass by finding nothing at all —
    /// which is the failure mode `run-all.sh`'s three-state model exists for,
    /// arriving inside a test instead of inside a script.
    #[test]
    fn the_reader_finds_every_arm_shape() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(arms.literals.contains("fx.literal"), "a plain literal arm");
        assert!(
            arms.literals.contains("fx.left"),
            "the left of an alternation"
        );
        assert!(
            arms.literals.contains("fx.right"),
            "the right of an alternation"
        );
        assert!(
            arms.literals.contains("fx.constant"),
            "a path pattern, resolved through the constant table — this is how \
             `crate::shell::commands::FILE_RECENT` is reached"
        );
        assert!(
            arms.guards.contains("fx_for_command"),
            "the guard's function"
        );
        assert!(arms.catch_all, "the catch-all");
    }

    /// **B. A planted unreachable command is reported.**
    ///
    /// The fixture is `CLEAN_FIXTURE` with the `"fx.literal"` arm deleted and
    /// nothing else changed — which is exactly the shape of the real defect:
    /// the command stays registered, the ribbon keeps drawing it, and only the
    /// arm is gone.
    #[test]
    fn a_deleted_arm_is_reported_unreachable() {
        let planted = CLEAN_FIXTURE.replace(r#"            "fx.literal" => self.one(),"#, "");
        assert_ne!(
            planted, CLEAN_FIXTURE,
            "the plant must actually change the fixture"
        );

        let before = fixture_arms(CLEAN_FIXTURE);
        let after = fixture_arms(&planted);
        assert!(
            is_routed("fx.literal", &before),
            "with its arm present the command must be reachable, or assertion B \
             proves nothing"
        );
        assert!(
            !is_routed("fx.literal", &after),
            "the reader did not notice a deleted arm — it cannot detect its own \
             planted violation, and its verdict on the real dispatcher is worth \
             nothing"
        );
        // …and only that one moved.
        assert!(is_routed("fx.left", &after));
        assert!(is_routed("fx.constant", &after));
    }

    /// **C. Neither a comment nor a string in an arm's body is an arm.**
    ///
    /// The two false passes a grep produces. `"fx.in_a_comment"` is a quoted id
    /// inside a `//` line — the exact shape of the doc comments in
    /// `app::dispatch`, which quote ids constantly — and `"fx.in_a_body"` is a
    /// string literal in executable code. A check that counted either would go
    /// green over a command whose arm had been deleted while the prose about it
    /// stayed.
    #[test]
    fn the_reader_does_not_see_comments_or_body_strings() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(
            !arms.literals.contains("fx.in_a_comment"),
            "a quoted id in a comment is not an arm"
        );
        assert!(
            !arms.literals.contains("fx.in_a_body"),
            "a string literal in an arm's body is not an arm"
        );
    }

    /// **D. A nested `match`'s arms are not the routing table's arms.**
    ///
    /// `dispatch_command` contains four nested `match` expressions inside arm
    /// bodies. A text scan cannot tell their arrows from the outer ones, and
    /// one of them has string literals on the right-hand side — so a grep would
    /// credit the dispatcher with routing ids it has never heard of. The tree
    /// walk visits the arms of one `match` and never descends into a body.
    #[test]
    fn the_reader_does_not_see_a_nested_match() {
        let arms = fixture_arms(CLEAN_FIXTURE);
        assert!(
            !arms.literals.contains("fx.in_a_nested_match"),
            "an arm of a `match` inside an arm's BODY routes nothing at the top \
             level, and crediting it is the false pass a grep produces"
        );
    }

    /// **E. An arm shape the reader does not understand is an error, not a
    /// shrug.**
    ///
    /// Failing closed is what keeps a future edit honest: a pattern nobody
    /// taught this reader about must stop the suite and be classified, rather
    /// than silently taking its ids out of the check.
    #[test]
    fn an_unreadable_arm_is_refused() {
        let odd = CLEAN_FIXTURE.replace(
            r#""fx.literal" => self.one(),"#,
            "crate::shell::commands::NOT_A_KNOWN_CONST => self.one(),",
        );
        let err = read_arms(&odd, &string_consts(FIXTURE_CONSTS))
            .expect_err("an unresolvable path pattern must be refused");
        assert!(
            err.contains("NOT_A_KNOWN_CONST"),
            "the error must name the arm: {err}"
        );
    }

    /// **F. A source with no dispatcher is refused rather than reported
    /// clean.**
    ///
    /// The "zero files scanned" failure, closed here as an `Err`. In the real
    /// module it cannot even arise: [`DISPATCH_SRC`] is an `include_str!` and a
    /// missing dispatcher file is a compile error.
    #[test]
    fn a_source_with_no_dispatcher_is_refused() {
        let err = read_arms("fn main() {}", &BTreeMap::new())
            .expect_err("a source with no dispatcher must not read as an empty routing table");
        assert!(
            err.contains(DISPATCHER),
            "the error must say what was missing: {err}"
        );
    }

    /// The constant reader resolves the shape an arm pattern can name.
    #[test]
    fn string_constants_are_resolved_from_their_defining_file() {
        let consts = string_consts(CONSTS_SRC);
        assert_eq!(
            consts.get("FILE_RECENT").map(String::as_str),
            Some(super::super::FILE_RECENT),
            "the reader must resolve `FILE_RECENT` to the same value Rust does, \
             or the one arm written as a constant reads as no arm at all"
        );
    }

    // -----------------------------------------------------------------
    // WHAT THE ALLOW-LIST SAYS ABOUT THE RIBBON
    // -----------------------------------------------------------------

    /// **★ How many drawn controls do nothing, and how many of those breach
    /// P3.**
    ///
    /// Not a rule — a **published number**, in the shape
    /// `the_icon_coverage_split_adds_up_to_the_registry` established: an
    /// arithmetic identity plus the two literals a reader actually consults, so
    /// that shortening this list is a visible act rather than a silent one.
    ///
    /// `RIBBON_IA.md` P3 says an unavailable capability renders **nothing**.
    /// Every entry marked `★ P3` in its reason is a control this module's
    /// author believes should not be drawn yet; removing one is a taxonomy
    /// decision and is the operator's. The count moving *down* is the project
    /// working.
    #[test]
    fn the_p3_tension_is_counted() {
        let total = SCAFFOLDED.len();
        let p3 = SCAFFOLDED
            .iter()
            .filter(|(_, reason)| reason.contains("\u{2605} P3"))
            .count();
        assert_eq!(
            total, 35,
            "the allow-list holds {total} entries; this module's header quotes the \
             figure, so move both together"
        );
        assert_eq!(
            p3, 8,
            "{p3} entries are marked as breaching P3 by being drawn at all; the \
             report to the operator quotes the figure, so move both together"
        );
        assert!(p3 <= total, "the P3 subset must be a subset");
        // ★ …and the mirror list's length, pinned for the same reason and in
        // the same place. It is **zero**: the four arms it used to tolerate
        // were deleted on 2026-08-15 after each verb was shown to have two
        // live routes that are not the dispatcher. A fifth dead arm is still
        // possible and still has to be argued — this assertion is what makes
        // adding one a visible act rather than a quiet one, and what stops the
        // header above going stale about it.
        assert_eq!(
            UNREACHED_ARMS.len(),
            0,
            "`UNREACHED_ARMS` is documented as empty. If an arm genuinely has \
             to be tolerated, add it there WITH its reason and move this number \
             — do not move this number alone."
        );
    }
}
