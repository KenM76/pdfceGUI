//! # `app::prefs::fonts` — where pdfce looks for a font it has to embed
//!
//! One preference, and it is the input two commands have been waiting for.
//!
//! ## ★★★ Why this exists, and why it is a PREFERENCE rather than a setting
//!
//! `tools.embed_fonts` and `tools.unembed_fonts` were registered, drawn on the
//! Tools tab and inert for the life of the project. Their recorded reason
//! quoted a premise that had expired — *"at S3 `Action` carries zoom and page
//! navigation and nothing else"* — and the entries themselves flagged it. But
//! re-deriving them on 2026-08-28 turned up a **second, unrecorded**
//! dependency that is the real one for embedding:
//!
//! `EmbedRequest::supplied` is a `/BaseFont` → donor-file map *"the shell
//! resolved for it"*, and `pdfce-cli`'s own note is blunt about the division of
//! labour: **"THE SOURCE FONTS COME FROM `--font-dir`. pdfce never goes
//! looking."** So there is nothing for an Embed command to send until an
//! operator has said where their fonts live.
//!
//! ★ That dependency was in neither register. It was found by asking what the
//! verb's own request struct requires, which is a different question from
//! *"does the verb exist"* — and the second question is the one the stale
//! blockers had all been answering.
//!
//! ## ★★ It lives in `userdata/preferences.txt`, not in `settings.txt`
//!
//! `crate::app::prefs`' header states the rule and it decides this cleanly:
//! `pdfce_core::settings` is for entries that **cite a clause the standard
//! leaves silent** — an ambiguity pdfce has to resolve one way or another.
//! *Where this operator keeps their font files* cites nothing. It is a fact
//! about a machine, and filing it there would make the settings window's own
//! opening paragraph dishonest.
//!
//! ## ★ Why a repeated key rather than one joined line
//!
//! `font_folder = C:\…` may appear as many times as the operator likes, and
//! every occurrence is another folder in search order. The alternative — a
//! separator-joined value — needs a separator that cannot occur in a path, and
//! on Windows the obvious candidates are all legal in one. A repeated key has
//! no such question, reads correctly in a file an operator edits by hand, and
//! makes "search order" visible as line order.
//!
//! ★ **Order is preserved and duplicates are dropped.** Order matters because
//! two folders may hold the same face and the first one wins; duplicates are
//! dropped because a folder listed twice is a folder searched twice for the
//! same answer, and because an operator who adds the same folder from the
//! picker twice has not asked for anything.

use std::path::{Path, PathBuf};

/// The most folders this preference will hold.
///
/// ★ Sixteen, and the cap exists for the same reason every cap in this project
/// does — a bound is a decision and an unbounded list is a decision nobody
/// made. It is not a performance limit: an embed searches folders once per
/// missing face. It is a **legibility** limit, because a settings pane listing
/// forty directories has stopped being a setting and become a file manager,
/// and because a preferences file that has accumulated forty entries is one
/// nobody has pruned.
pub const MAX_FOLDERS: usize = 16;

/// Add `folder`, keeping order and refusing a duplicate or an over-long list.
///
/// Returns whether the list changed, so a caller can tell "added" from "you
/// already have that one" without comparing lengths.
///
/// ★ It does **not** check that the folder exists. A removable drive that is
/// not mounted right now is still where the operator's fonts live, and a
/// preference that silently dropped it on the day the drive was unplugged
/// would be worse than one that keeps a path that occasionally resolves to
/// nothing. The *embed* is where a missing folder is reported, because that is
/// where it matters.
pub fn add(folders: &mut Vec<PathBuf>, folder: &Path) -> bool {
    if folders.len() >= MAX_FOLDERS || folders.iter().any(|f| f == folder) {
        return false;
    }
    folders.push(folder.to_path_buf());
    true
}

/// Parse one `font_folder = …` line's value.
///
/// ★ Trims, and rejects only the empty result. A path is otherwise taken
/// verbatim — no canonicalisation, no separator normalisation — because
/// `Path` comparison on Windows is case-insensitive in the filesystem and
/// case-sensitive in `PathBuf`, and a preference that rewrote what the
/// operator typed would make their own file unrecognisable to them.
#[must_use]
pub fn parse_one(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

/// The `font_folder` lines for [`super::Prefs::write_to_string`], with the
/// comment that explains them.
///
/// ★ The comment is emitted **even when the list is empty**, which is the
/// convention every other block in that file follows and is the reason the
/// file is editable by hand: an operator who wants to add a folder without
/// opening pdfce needs to see the key name and its rules, and a key that only
/// appears once it is already set cannot teach anybody anything.
#[must_use]
pub fn write_block(folders: &[PathBuf]) -> String {
    let mut out = String::from(
        "\n\
         # Folders pdfce searches when it has to embed a font that a document\n\
         # names but does not carry. Repeat the key for more than one; they are\n\
         # searched in the order they appear here. Up to 16.\n\
         #\n\
         # pdfce never goes looking on its own -- if this is empty, embedding\n\
         # has nowhere to take a font from.\n",
    );
    if folders.is_empty() {
        // ui-text-exempt: a file KEY inside a commented example line.
        out.push_str("# font_folder = C:\\Windows\\Fonts\n");
        return out;
    }
    for folder in folders {
        // ui-text-exempt: a file KEY, never displayed in the UI.
        out.push_str("font_folder = ");
        out.push_str(&folder.display().to_string());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A duplicate is refused and the list is unchanged.**
    ///
    /// ★ Asserted through the return value as well as the length, because the
    /// caller uses it to decide what to say: *"added"* and *"you already have
    /// that one"* are different sentences and a length comparison cannot tell
    /// them apart when the list is also at its cap.
    #[test]
    fn a_duplicate_is_refused_and_says_so() {
        let mut folders = Vec::new();
        assert!(add(&mut folders, Path::new("C:/Fonts")));
        assert!(!add(&mut folders, Path::new("C:/Fonts")));
        assert_eq!(folders.len(), 1);
    }

    /// **Order is preserved**, because it is search order and the first match
    /// wins.
    #[test]
    fn order_is_search_order() {
        let mut folders = Vec::new();
        add(&mut folders, Path::new("C:/First"));
        add(&mut folders, Path::new("C:/Second"));
        assert_eq!(folders[0], PathBuf::from("C:/First"));
        assert_eq!(folders[1], PathBuf::from("C:/Second"));
    }

    /// **The cap holds**, and the seventeenth is refused rather than evicting
    /// the first — an operator who has hit the limit is told, not silently
    /// rearranged.
    #[test]
    fn the_cap_refuses_rather_than_evicting() {
        let mut folders = Vec::new();
        for i in 0..MAX_FOLDERS {
            assert!(add(&mut folders, &PathBuf::from(format!("C:/F{i}"))));
        }
        assert!(!add(&mut folders, Path::new("C:/OneMore")));
        assert_eq!(folders.len(), MAX_FOLDERS);
        assert_eq!(folders[0], PathBuf::from("C:/F0"), "the first survives");
    }

    /// **An empty or blank value is not a folder.**
    #[test]
    fn a_blank_value_is_not_a_path() {
        assert!(parse_one("").is_none());
        assert!(parse_one("   ").is_none());
        assert_eq!(parse_one("  C:/Fonts  "), Some(PathBuf::from("C:/Fonts")));
    }

    /// ★★ **The comment block is written even with no folders**, so the file
    /// teaches its own key.
    ///
    /// The failure this guards is the tempting simplification — emit nothing
    /// when the list is empty — which produces a preferences file with no
    /// mention of the one key an operator would want to add by hand.
    #[test]
    fn the_key_is_documented_even_when_unset() {
        let block = write_block(&[]);
        assert!(block.contains("font_folder"), "the key is named: {block}");
        assert!(
            block.contains("never goes looking"),
            "and the consequence of leaving it empty is stated: {block}"
        );
    }

    /// **A written list round-trips through the parser.**
    #[test]
    fn a_written_list_reads_back() {
        let folders = vec![PathBuf::from("C:/A"), PathBuf::from("D:/B")];
        let block = write_block(&folders);
        let read: Vec<PathBuf> = block
            .lines()
            .filter_map(|l| l.strip_prefix("font_folder = "))
            .filter_map(parse_one)
            .collect();
        assert_eq!(read, folders);
    }
}
