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
//! ## ★★ Resolution is the shell's and it is not a guess
//!
//! `pdfce_render::font::program::FontProgram::parse` reads a font file's own
//! advertised names, so a donor is matched on **what the file says it is**
//! rather than on what somebody called the file. The filename stem is
//! registered as well — `pdfce-cli` does the same and for the stated reason:
//! *"so a match works even when the internal name is odd or absent"* — but it
//! is a fallback, and [`Donor::matched`] records which of the two answered so
//! the operator can be told.
//!
//! ★ That distinction is the whole reason `FontMatch` exists in the engine's
//! request. An `Exact` match is the file agreeing with the document; an `Alias`
//! is this shell deciding two names mean the same face, which is an inference
//! and owes a disclosure.
//!
//! ## Determinism
//!
//! Folders are searched in list order and files within a folder are **sorted**
//! before being read, so two runs over the same folders produce the same
//! donor for the same face. `pdfce-cli` sorts for the same reason and cites
//! R19's spirit: an OS directory-iteration order is not an order.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

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

/// One font file that could stand in for a face a document is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Donor {
    /// Where it came from, for the disclosure and for the engine's
    /// `SuppliedFont::source`.
    pub path: PathBuf,
    /// The name that matched — the file's own, or its filename stem.
    pub face_name: String,
    /// **How** it matched, which the operator is owed.
    ///
    /// ★ `Exact` is the file's advertised name equalling the document's
    /// `/BaseFont` with its §9.6.4 subset tag stripped. Anything else is this
    /// shell deciding two names mean one face, which is an inference — and
    /// Rule 4's surviving half says an inference the operator cannot see still
    /// owes them an off-canvas report.
    pub matched: Match,
}

/// How a donor was matched to a face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// The file's own advertised name equals the document's, tag stripped.
    Exact,
    /// The file's **filename stem** matched where its advertised names did
    /// not. A weaker answer, and named separately so it can be disclosed.
    Stem,
}

/// Everything the configured folders offer, indexed by every name each file
/// answers to.
///
/// ★ A `BTreeMap` rather than a `HashMap`: the iteration order is stable, and
/// this is read to build a report an operator compares between runs.
#[derive(Debug, Default)]
pub struct Library {
    by_name: BTreeMap<String, Donor>,
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
    /// The first folder holding a name keeps it, which is the opposite of
    /// `pdfce-cli`'s renderer environment (*"duplicate-name precedence: last
    /// wins"*) and is deliberate. That environment is built once per render
    /// and the last registration is simply the surviving one; **this** list is
    /// the operator's, in an order they typed, and the Settings hint says
    /// *"searched in the order they appear here"*. First-wins is what makes
    /// that sentence true.
    #[must_use]
    pub fn scan(folders: &[PathBuf]) -> Self {
        let mut library = Self::default();
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
        for name in &names {
            self.offer(name, path, Match::Exact);
        }
        // ★★ The filename stem, as a FALLBACK and recorded as one.
        // `pdfce-cli` registers it too — *"so a match works even when the
        // internal name is odd or absent"* — and the difference here is that
        // this shell has to tell an operator which happened, because a stem
        // match is this program deciding a file called `Helv.ttf` is
        // Helvetica.
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && !names.iter().any(|n| n == stem)
        {
            self.offer(stem, path, Match::Stem);
        }
        if names.is_empty() && path.file_stem().is_none() {
            self.skipped.push(crate::text::fonts::no_name(path));
        }
    }

    /// Record `name` → this file, unless an earlier folder already claimed it.
    fn offer(&mut self, name: &str, path: &Path, matched: Match) {
        self.by_name
            .entry(name.to_owned())
            .or_insert_with(|| Donor {
                path: path.to_path_buf(),
                face_name: name.to_owned(),
                matched,
            });
    }

    /// The donor for a document's `/BaseFont`, if the folders hold one.
    ///
    /// ★★ The subset tag is stripped before matching, and it has to be: a
    /// §9.6.4 tag is six uppercase letters and a `+`, minted per subset, so
    /// `ABCDEF+ArialMT` and `GHIJKL+ArialMT` are the same face and neither is a
    /// name any font file advertises. Matching without stripping would find
    /// nothing, ever, on exactly the documents that need embedding most.
    #[must_use]
    pub fn donor_for(&self, base_font: &str) -> Option<&Donor> {
        self.by_name.get(strip_subset_tag(base_font))
    }

    /// How many distinct names the folders answer to.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_name.len()
    }

    /// Whether the folders offered nothing at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_name.is_empty()
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

    /// ★★ **The first folder holding a name keeps it.**
    ///
    /// The opposite of the renderer environment's last-wins, and deliberately:
    /// the Settings hint promises *"searched in the order they appear here"*,
    /// and first-wins is what makes that sentence true. Asserted directly on
    /// `offer`, because building it from real files would need two font files
    /// on disk and would be testing the filesystem.
    #[test]
    fn the_first_folder_to_offer_a_name_keeps_it() {
        let mut library = Library::default();
        library.offer("ArialMT", Path::new("C:/first/Arial.ttf"), Match::Exact);
        library.offer("ArialMT", Path::new("C:/second/Arial.ttf"), Match::Exact);
        let donor = library.donor_for("ArialMT").expect("indexed");
        assert_eq!(donor.path, PathBuf::from("C:/first/Arial.ttf"));
    }

    /// **A tagged `/BaseFont` finds an untagged donor.**
    ///
    /// The case that matters on real documents: a subsetted face is what needs
    /// embedding, and its name never matches a font file's.
    #[test]
    fn a_subsetted_base_font_finds_its_donor() {
        let mut library = Library::default();
        library.offer("ArialMT", Path::new("C:/f/Arial.ttf"), Match::Exact);
        assert!(library.donor_for("ABCDEF+ArialMT").is_some());
        assert!(library.donor_for("ArialMT").is_some());
        assert!(library.donor_for("TimesNewRomanPSMT").is_none());
    }

    /// **A stem match is recorded as one**, so it can be disclosed.
    #[test]
    fn a_stem_match_is_distinguishable_from_an_exact_one() {
        let mut library = Library::default();
        library.offer("Helvetica", Path::new("C:/f/Helvetica.ttf"), Match::Exact);
        library.offer("Helv", Path::new("C:/f/Helv.ttf"), Match::Stem);
        assert_eq!(
            library.donor_for("Helvetica").unwrap().matched,
            Match::Exact
        );
        assert_eq!(library.donor_for("Helv").unwrap().matched, Match::Stem);
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
    /// whose parser never ran. `FontProgram::parse` is the one link this module
    /// does not own and cannot fake, and *"the folders yielded nothing"* is
    /// indistinguishable from *"the folders were empty"* without a folder that
    /// is not.
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
        // ★ A name every Windows machine carries, matched the way a document
        // would spell it. Asserting a SPECIFIC face rather than a count is what
        // makes this a test of the join rather than of `read_dir`.
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
        assert!(
            library.donor_for("ABCDEF+ArialMT").is_some() || library.donor_for("Arial").is_some(),
            "neither `Arial` nor a subsetted `ArialMT` resolved out of {} indexed name(s)",
            library.len()
        );
    }
}
