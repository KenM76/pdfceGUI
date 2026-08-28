//! # `app::fonts` — turning the operator's font folders into donors an embed
//! can use
//!
//! The half of font embedding that is **the shell's**, and the engine is
//! explicit that it is: `EmbedRequest::supplied` is a `/BaseFont` → donor map
//! *"the shell resolved for it"*, and `pdfce-cli`'s own note is blunter —
//! **"the source fonts come from `--font-dir`; pdfce never goes looking."**
//!
//! ## ★★★ Why pdfce does not go looking, and why this module must not either
//!
//! Embedding puts a font **program** — the actual outlines — inside somebody's
//! document, which they then send to somebody else. Which font that is, is a
//! licensing question with a different answer for every foundry, and a program
//! that searched `C:\Windows\Fonts` on its own would be answering it silently
//! on the operator's behalf, in a file that outlives the decision.
//!
//! So the folders come from [`crate::app::prefs::Prefs::font_folders`], which
//! is empty until an operator puts something in it, and this module searches
//! **those and nothing else**. There is no fallback, no bundled directory, and
//! no "well, try the system fonts too". An empty list means an embed has
//! nowhere to take a font from, and that is reported rather than worked around.
//!
//! ## ★★★ The WALK is the shell's; the RESOLUTION is the engine's
//!
//! This module reads files. `pdfce_render::FontEnvironment` decides what
//! answers to what. The split is not tidiness — it is that the second half was
//! **already written**, three rungs deep and doctested, and the first draft of
//! this module reimplemented the shallowest of them.
//!
//! `resolve_for_embedding` tries, in order:
//!
//! | rung | what it means |
//! |---|---|
//! | `Exact` | a file advertising the name the document spells, tag stripped |
//! | `Alias` | a **standard-14 family equivalence** — `Helvetica` → `Arial` |
//! | `Bundled` | a face pdfce itself ships. **Not offered here** — see below |
//!
//! ★★ The middle rung is what makes this feature work at all on this platform.
//! Every CAD drawing this project exists for asks for `Helvetica`, and **no
//! Windows machine has a font called Helvetica**. A resolver with only the
//! first rung finds nothing on exactly the documents that need embedding most
//! — which is what the first draft of this module did, on the day it was
//! written, and it is why the delegation is worth the crate dependency.
//!
//! ⇒ Recorded because the shape generalises: *"the shell owns resolution"* was
//! read as *"the shell must implement resolution"*. It means the shell owns the
//! **filesystem** — `pdfce-core` must not read a directory — and `pdfce-render`
//! is not `pdfce-core`.
//!
//! ## ★★ Why bundled faces are NOT offered
//!
//! The third rung would supply one of pdfce's own standard-14 substitutes with
//! no folder configured at all, and it is passed `false` here. `pdfce-cli`
//! makes it opt-in behind `--use-bundled-fonts`; this shell has no equivalent
//! switch, because the Embed window has no settings by design.
//!
//! ★ That is an **operator decision this session did not take** — filed in
//! `OPERATOR_REQUESTS.md` — rather than a limitation being papered over.
//! Embedding a substitute face changes what the letters look like in a document
//! somebody sends out, and offering it silently because it happened to be
//! available is precisely the *sneaky* half of rule 4.
//!
//! ## ★★ A stem match is still disclosed, and the engine does not distinguish it
//!
//! `FontEnvironment` registers a file's filename stem alongside its advertised
//! names and reports a hit on either as `Exact`. That is right for the
//! *renderer*, where the question is only *"can I draw this"*. It is not enough
//! here: a stem match is this shell deciding that a file called `Helv.ttf` is
//! the face a document spells `Helvetica`, which is an inference, and rule 4's
//! surviving half says an inference the operator cannot see owes them a report.
//!
//! So [`Library`] keeps its own record of which names came only from a stem and
//! re-grades such a hit to [`Match::Stem`]. **The engine is told `Alias`** for
//! one — see `dialogs::embed`, which explains why understating it would disable
//! the engine's symbolic guard from the outside.
//!
//! ## Determinism
//!
//! Folders are searched in list order and files within a folder are **sorted**
//! before being read, so two runs over the same folders produce the same donor
//! for the same face. `pdfce-cli` sorts for the same reason and cites R19's
//! spirit: an OS directory-iteration order is not an order.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use pdfce_render::FontEnvironment;

/// The largest font file this will read, in bytes.
///
/// ★ Sixteen mebibytes, matching `pdfce-cli`'s own ceiling. It is not about
/// memory — it is that a "font file" above this size is nearly always
/// something else that happens to have a font extension, and reading it costs
/// an operator a visible pause for an answer that will be *"not a usable
/// font"*. The skip is reported rather than silent.
pub const MAX_FONT_FILE_BYTES: u64 = 16 * 1024 * 1024;

/// The extensions this will attempt.
///
/// ★ `.ttc` and `.otc` are **deliberately absent**. A collection holds several
/// faces in one file and the engine refuses one outright
/// (`EmbedBlocker::ProgramIsCollection`), so offering one as a donor would be
/// resolving a face to a file that is then refused by name — a press that
/// always fails, which is what this project spends its time removing.
const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "pfb", "cff"];

/// One font program that can stand in for a face a document is missing.
///
/// Borrows from the [`Library`] that resolved it. The bytes are held once, in
/// the environment; a donor that owned a copy would clone a whole face on every
/// lookup, for a value most callers only read a name out of.
#[derive(Debug, Clone, Copy)]
pub struct Donor<'a> {
    /// The file it came from, for the disclosure and for the engine's
    /// `SuppliedFont::source`.
    pub path: &'a Path,
    /// The name that matched — the document's own, an equivalent family's, or
    /// the file's stem.
    pub face_name: &'a str,
    /// The program bytes, exactly as they will be embedded.
    pub program: &'a [u8],
    /// **How** it matched, which the operator is owed.
    pub matched: Match,
}

/// How a donor was matched to a face.
///
/// ★ `pdfce_render::font::EmbedMatch`'s three rungs minus its bundled one, plus
/// a distinction of this shell's own. See the module header for why `Stem`
/// exists here and not there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// A file advertising the name the document spells, tag stripped.
    Exact,
    /// A **standard-14 family equivalence** — the document says `Helvetica` and
    /// the folder holds `Arial`. Metric-compatible by design, and the advances
    /// come from `/Widths` regardless, but the letterforms are a different
    /// designer's.
    Alias,
    /// The file's **filename stem** matched where its advertised names did not.
    /// The weakest answer, and named separately so it can be disclosed.
    Stem,
}

impl Match {
    /// Whether this is something other than the face the document named.
    #[must_use]
    pub const fn is_inferred(self) -> bool {
        !matches!(self, Self::Exact)
    }
}

/// Everything the configured folders offer.
///
/// ★ The names live in a `FontEnvironment` — which owns the bytes and answers
/// the three-rung question — and the **paths** live here, because the
/// environment has no notion of where a face came from and the operator is owed
/// exactly that.
#[derive(Debug, Default)]
pub struct Library {
    /// The engine's resolver, populated by [`Self::scan`].
    env: FontEnvironment,
    /// Every registered name → the file it came from.
    ///
    /// ★ A `BTreeMap` rather than a `HashMap`: iteration order is stable, and
    /// this is read to build a report an operator compares between runs.
    paths: BTreeMap<String, PathBuf>,
    /// The names that came **only** from a filename stem. See the header.
    stems: BTreeSet<String>,
    /// Files that were skipped and why, in the order they were met.
    ///
    /// ★ Kept rather than discarded, because *"pdfce could not embed
    /// HelveticaNeue"* and *"pdfce skipped HelveticaNeue.ttf because it is 40
    /// MB"* are the same event to the program and completely different events
    /// to the operator. The second is actionable.
    pub skipped: Vec<String>,
}

impl Library {
    /// Read every font file in `folders`, in order, and index what they offer.
    ///
    /// # ★★ Later folders do NOT win
    ///
    /// The first folder holding a name keeps it, which is the opposite of the
    /// renderer environment's own precedence (*"duplicate-name precedence: last
    /// wins"*) and is deliberate. That environment is built once per render and
    /// the last registration is simply the surviving one; **this** list is the
    /// operator's, in an order they typed, and the Settings hint promises
    /// *"searched in the order they appear here"*. First-wins is what makes
    /// that sentence true.
    ///
    /// ★ Enforced by [`Self::offer`] rather than by registration order, because
    /// `FontEnvironment::insert_named` is last-wins and would silently reverse
    /// it.
    #[must_use]
    pub fn scan(folders: &[PathBuf]) -> Self {
        // ★ `bundled()` rather than an empty environment, and it is safe: the
        // bundled faces live in the FALLBACK table, which
        // `resolve_for_embedding` consults only when it is passed
        // `allow_bundled`. It is passed `false` in [`Self::donor_for`], every
        // time, for the header's reason — and there is a test that presses on
        // exactly that.
        let mut library = Self {
            env: FontEnvironment::bundled(),
            ..Self::default()
        };
        for folder in folders {
            library.scan_one(folder);
        }
        library
    }

    fn scan_one(&mut self, folder: &Path) {
        let entries = match std::fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(error) => {
                // ★ A folder that will not open is a **note, not a failure**.
                // A removable drive that is not mounted is still where the
                // operator's fonts live — `prefs::fonts::add`'s stated position
                // — so the honest response is to say so and search the rest.
                self.skipped.push(crate::text::fonts::folder_unreadable(
                    folder,
                    &error.to_string(),
                ));
                return;
            }
        };
        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_file() && has_font_extension(path))
            .collect();
        // See the module header on determinism.
        files.sort();
        for path in files {
            self.read_one(&path);
        }
    }

    fn read_one(&mut self, path: &Path) {
        if let Ok(meta) = std::fs::metadata(path)
            && meta.len() > MAX_FONT_FILE_BYTES
        {
            self.skipped
                .push(crate::text::fonts::file_too_large(path, meta.len()));
            return;
        }
        let Ok(bytes) = std::fs::read(path) else {
            self.skipped.push(crate::text::fonts::file_unreadable(path));
            return;
        };
        // ★ Parsed ONCE, and the borrow ends before the bytes are stored —
        // `pdfce-cli` notes the same discipline against R21. A second parse to
        // re-read a name would double the cost of a scan over a system font
        // folder, which is the case this is most likely to meet.
        let names = match pdfce_render::font::program::FontProgram::parse(&bytes) {
            Ok(program) => program.face_names(),
            Err(error) => {
                self.skipped
                    .push(crate::text::fonts::not_a_font(path, &error.to_string()));
                return;
            }
        };
        let stem = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned);
        if names.is_empty() && stem.is_none() {
            self.skipped.push(crate::text::fonts::no_name(path));
            return;
        }
        // ★★ The bytes are wrapped ONCE and every `FontData` clone below is an
        // `Arc` clone. A file advertising four names would otherwise be four
        // full copies of one face in memory, and a system font folder holds
        // thousands of files.
        let data = pdfce_render::FontData::new(bytes);
        for name in &names {
            self.offer(name, path, &data, false);
        }
        // ★★ The filename stem, as a FALLBACK and recorded as one.
        // `pdfce-cli` registers it too — *"so a match works even when the
        // internal name is odd or absent"* — and the difference here is that
        // this shell has to tell an operator which happened, because a stem
        // match is this program deciding a file called `Helv.ttf` is Helvetica.
        if let Some(stem) = stem
            && !names.contains(&stem)
        {
            self.offer(&stem, path, &data, true);
        }
    }

    /// Record `name` → this file, unless an earlier folder already claimed it.
    fn offer(&mut self, name: &str, path: &Path, data: &pdfce_render::FontData, from_stem: bool) {
        if self.paths.contains_key(name) {
            return;
        }
        self.paths.insert(name.to_owned(), path.to_path_buf());
        if from_stem {
            self.stems.insert(name.to_owned());
        }
        self.env.insert_named(name, data.clone());
    }

    /// The donor for a document's `/BaseFont`, if the folders hold one.
    ///
    /// ★★ The subset tag is handled by the engine — `resolve_for_embedding`
    /// strips it on its second rung — and it has to be: a §9.6.4 tag is six
    /// uppercase letters and a `+`, minted per subset, so `ABCDEF+ArialMT` and
    /// `GHIJKL+ArialMT` are the same face and neither is a name any font file
    /// advertises. Matching without stripping would find nothing, ever, on
    /// exactly the documents that need embedding most.
    ///
    /// ★ `allow_bundled: false`, always. See the module header.
    #[must_use]
    pub fn donor_for(&self, base_font: &str) -> Option<Donor<'_>> {
        let hit = self.env.resolve_for_embedding(base_font, false)?;
        let (name, path) = self.paths.get_key_value(hit.face_name.as_str())?;
        let matched = match hit.quality {
            // ★ The re-grade the header explains. The engine says `Exact` for a
            // stem hit because to a renderer the two are the same question; to
            // a disclosure they are not.
            pdfce_render::font::EmbedMatch::Exact if self.stems.contains(name) => Match::Stem,
            pdfce_render::font::EmbedMatch::Exact => Match::Exact,
            // `Bundled` is unreachable — `allow_bundled` is `false` — and it is
            // folded in with `Alias` rather than given an arm that claims to
            // handle it. Both mean "not the face the document named", which is
            // the only thing a caller does with this value.
            _ => Match::Alias,
        };
        Some(Donor {
            path,
            face_name: name,
            program: hit.data.bytes(),
            matched,
        })
    }

    /// How many distinct names the folders answer to.
    #[must_use]
    pub fn len(&self) -> usize {
        self.paths.len()
    }

    /// Whether the folders offered nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }
}

/// Whether the path's extension is one this will attempt.
fn has_font_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .is_some_and(|e| FONT_EXTENSIONS.contains(&e.as_str()))
}

/// A `/BaseFont` without its §9.6.4 subset tag.
///
/// ★ Exactly six uppercase letters and a `+`, per the standard. Anything else
/// before a `+` is part of the name and is kept — `Foo+Bar` is a legal, if
/// unusual, font name, and treating it as a tag would look for a face called
/// `Bar`.
///
/// ★★ Kept even though [`Library::donor_for`] no longer calls it: the
/// **display** side needs it, because a row reading `ABCDEF+ArialMT` shows an
/// operator a tag that is an artefact of subsetting and means nothing to them.
/// Resolution and presentation happen to want the same rule, and only one of
/// them is the engine's.
#[must_use]
pub fn strip_subset_tag(base_font: &str) -> &str {
    match base_font.split_once('+') {
        Some((tag, rest)) if tag.len() == 6 && tag.bytes().all(|b| b.is_ascii_uppercase()) => rest,
        _ => base_font,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A four-byte non-font, for the tests that only exercise the index.
    ///
    /// ★ `FontEnvironment::insert_named` does not parse, so a donor can be
    /// registered without a real face. What that does **not** buy is coverage
    /// of the parse — see [`real_files`], which exists precisely because every
    /// test in this module would pass on a build whose parser was dead.
    fn stub() -> pdfce_render::FontData {
        pdfce_render::FontData::new(vec![0u8; 4])
    }

    /// **A subset tag is stripped and nothing else is.**
    ///
    /// ★ The negative cases are the point. A five-letter prefix, a lowercase
    /// one, and a name that simply contains a `+` are all names in their own
    /// right, and treating any of them as a tag would search for a face that
    /// does not exist — silently, since the result is just "no donor".
    #[test]
    fn only_a_real_subset_tag_is_stripped() {
        assert_eq!(strip_subset_tag("ABCDEF+ArialMT"), "ArialMT");
        assert_eq!(strip_subset_tag("ArialMT"), "ArialMT");
        assert_eq!(strip_subset_tag("ABCDE+ArialMT"), "ABCDE+ArialMT");
        assert_eq!(strip_subset_tag("abcdef+ArialMT"), "abcdef+ArialMT");
        assert_eq!(strip_subset_tag("Foo+Bar"), "Foo+Bar");
    }

    /// **Only real font extensions are attempted.**
    ///
    /// ★★ `.ttc` and `.otc` must stay out, and that is a capability decision
    /// rather than an oversight: the engine refuses a collection by name
    /// (`EmbedBlocker::ProgramIsCollection`), so offering one as a donor would
    /// resolve a face to a file guaranteed to be rejected — a press that always
    /// fails.
    #[test]
    fn a_collection_is_not_offered_as_a_donor() {
        assert!(has_font_extension(Path::new("C:/f/Arial.ttf")));
        assert!(has_font_extension(Path::new("C:/f/Arial.OTF")));
        assert!(!has_font_extension(Path::new("C:/f/Cambria.ttc")));
        assert!(!has_font_extension(Path::new("C:/f/Cambria.otc")));
        assert!(!has_font_extension(Path::new("C:/f/readme.txt")));
        assert!(!has_font_extension(Path::new("C:/f/Arial")));
    }

    /// ★★★ **The first folder holding a name keeps it.**
    ///
    /// The opposite of the renderer environment's last-wins, and the assertion
    /// is load-bearing rather than decorative **now that this delegates to that
    /// environment**: `insert_named` *is* last-wins, and nothing but
    /// [`Library::offer`]'s guard stops the operator's stated search order from
    /// being silently reversed by the crate underneath.
    #[test]
    fn the_first_folder_to_offer_a_name_keeps_it() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/first/Arial.ttf"), &stub(), false);
        library.offer("ArialMT", Path::new("C:/second/Arial.ttf"), &stub(), false);
        let donor = library.donor_for("ArialMT").expect("indexed");
        assert_eq!(donor.path, Path::new("C:/first/Arial.ttf"));
    }

    /// **A tagged `/BaseFont` finds an untagged donor.**
    ///
    /// The case that matters on real documents: a subsetted face is what needs
    /// embedding, and its name never matches a font file's.
    #[test]
    fn a_subsetted_base_font_finds_its_donor() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), &stub(), false);
        assert!(library.donor_for("ABCDEF+ArialMT").is_some());
        assert!(library.donor_for("ArialMT").is_some());
        assert!(library.donor_for("Wingdings").is_none());
    }

    /// ★★★ **`Helvetica` resolves to Arial, and it is graded as a substitute.**
    ///
    /// **The test this rewrite exists for.** No Windows machine has a font
    /// called Helvetica, and every CAD drawing this project serves asks for
    /// one, so a resolver with only an exact rung finds nothing on precisely
    /// the documents that need embedding most. That is what the first draft of
    /// this module did — and no test in it could see the gap, because every one
    /// of them registered a name and then asked for that same name.
    #[test]
    fn helvetica_finds_arial_and_says_it_is_a_substitute() {
        let mut library = Library::scan(&[]);
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), &stub(), false);
        let donor = library.donor_for("Helvetica").expect("the alias rung");
        assert_eq!(donor.face_name, "ArialMT");
        assert_eq!(donor.matched, Match::Alias);
        assert!(donor.matched.is_inferred());
    }

    /// ★★ **A stem match is re-graded, where the engine calls it exact.**
    ///
    /// The engine registers a filename stem beside a file's advertised names
    /// and reports either as `Exact`, which is right for a renderer and not
    /// enough for a disclosure. Losing this would tell an operator that pdfce
    /// found the face their document named, when what it found was a file with
    /// a suggestive name.
    #[test]
    fn a_stem_match_is_distinguishable_from_an_exact_one() {
        let mut library = Library::scan(&[]);
        library.offer("Helvetica", Path::new("C:/f/Helvetica.ttf"), &stub(), false);
        library.offer("Helv", Path::new("C:/f/Helv.ttf"), &stub(), true);
        assert_eq!(
            library.donor_for("Helvetica").expect("exact").matched,
            Match::Exact
        );
        assert_eq!(
            library.donor_for("Helv").expect("stem").matched,
            Match::Stem
        );
    }

    /// ★★ **A bundled face is never offered.**
    ///
    /// `FontEnvironment::bundled()` is what this scans into, so pdfce's own
    /// standard-14 substitutes sit in the table the whole time and one `true`
    /// would put them into somebody's document without a folder being
    /// configured at all. Whether to offer them is an operator decision, and
    /// this is the assertion that keeps it from being taken by accident.
    #[test]
    fn nothing_resolves_out_of_an_empty_library() {
        let library = Library::scan(&[]);
        assert!(library.is_empty());
        assert!(
            library.donor_for("Helvetica").is_none(),
            "a bundled substitute was offered with no folder configured"
        );
        assert!(library.donor_for("Times-Roman").is_none());
        assert!(library.donor_for("Courier").is_none());
    }

    /// **A folder that will not open is a note, not a panic and not a stop.**
    ///
    /// ★ The remaining folders are still searched. An operator with a removable
    /// drive in their list has one folder that comes and goes, and a scan that
    /// abandoned the rest of the list when it met one would make the feature
    /// unreliable in a way they could not diagnose.
    #[test]
    fn an_unreadable_folder_is_noted_and_the_rest_are_searched() {
        let library = Library::scan(&[
            PathBuf::from("C:/definitely/not/here/at/all"),
            PathBuf::from("C:/nor/this/one"),
        ]);
        assert_eq!(
            library.skipped.len(),
            2,
            "both were noted: {:?}",
            library.skipped
        );
        assert!(library.is_empty());
    }
}

#[cfg(test)]
mod real_files {
    use super::*;

    /// ★★★ **The scan reads a real font folder and finds real faces.**
    ///
    /// Every test above is about the INDEX — the tag rule, first-wins, which
    /// extensions are attempted — and every one of them would pass on a build
    /// whose parser never ran, because they register a stub and then ask for
    /// it. `FontProgram::parse` is the one link this module does not own and
    /// cannot fake, and *"the folders yielded nothing"* is indistinguishable
    /// from *"the folders were empty"* without a folder that is not.
    ///
    /// ★ It uses the operating system's own font directory, which is the one
    /// folder that certainly exists on the machine this ships for — and is
    /// deliberately **not** what the product searches: `Prefs::font_folders`
    /// starts empty and this module never adds to it, for the licensing reason
    /// in the header. A test may look where a product may not.
    ///
    /// SKIPPED rather than failed where that directory is absent, because its
    /// absence is a fact about the machine and not about this code.
    #[test]
    fn a_real_font_folder_yields_real_faces() {
        let dir = PathBuf::from(r"C:\Windows\Fonts");
        if !dir.is_dir() {
            eprintln!("no system font directory on this machine — skipped");
            return;
        }
        let library = Library::scan(&[dir]);
        assert!(
            !library.is_empty(),
            "a system font folder yielded no faces at all, which means the parse link is dead. \
             Skips: {:?}",
            library.skipped.iter().take(5).collect::<Vec<_>>()
        );
        // ★ Printed rather than asserted on. Measured on the development
        // machine at **3,359 indexed names from one skip**, which is the number
        // that made this test evidence rather than a green tick — a build whose
        // parser was dead would index the filename stems alone and still be
        // "not empty". Not asserted, because it is a fact about somebody's
        // Windows install and would pin this test to a machine.
        eprintln!(
            "indexed {} name(s), {} skip(s)",
            library.len(),
            library.skipped.len()
        );
        // ★ A name every Windows machine carries, matched the way a document
        // would spell it. Asserting a SPECIFIC face rather than a count is what
        // makes this a test of the join rather than of `read_dir`.
        assert!(
            library.donor_for("ABCDEF+ArialMT").is_some() || library.donor_for("Arial").is_some(),
            "neither `Arial` nor a subsetted `ArialMT` resolved out of {} indexed name(s)",
            library.len()
        );
        // ★★★ **The claim this whole rewrite rests on, on a real machine.**
        //
        // `Helvetica` is what the fixture asks for and what every CAD exporter
        // writes; nothing on Windows advertises that name. If the alias rung
        // works, this resolves to an Arial out of the system folder and reports
        // itself as a substitute. If it does not, embedding a CAD drawing on
        // this platform does nothing at all — which is the state this module
        // shipped in for exactly one commit.
        let donor = library
            .donor_for("Helvetica")
            .expect("no donor for Helvetica out of a real Windows font folder");
        assert!(
            donor.matched.is_inferred(),
            "a real Helvetica was found, which no Windows machine has — the grading is wrong"
        );
        eprintln!(
            "Helvetica -> {} ({:?}) from {}",
            donor.face_name,
            donor.matched,
            donor.path.display()
        );
    }
}
