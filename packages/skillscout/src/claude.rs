use std::fs;
use std::path::Path;

pub const SECTION_START: &str = "<!-- skillscout:start -->";
pub const SECTION_END: &str = "<!-- skillscout:end -->";
pub const LEGACY_SECTION_START: &str = "<!-- autoskills:start -->";
pub const LEGACY_SECTION_END: &str = "<!-- autoskills:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupResult {
    pub cleaned: bool,
    pub deleted: bool,
}

/// Mirrors `cleanupClaudeMd` in claude.ts
/// Strips `SECTION_START`/`SECTION_END` (or legacy) and deletes file if only `# CLAUDE.md` remains.
pub fn cleanup_claude_md(project_dir: &Path) -> CleanupResult {
    let output_path = project_dir.join("CLAUDE.md");

    if !output_path.exists() {
        return CleanupResult {
            cleaned: false,
            deleted: false,
        };
    }

    let existing = match fs::read_to_string(&output_path) {
        Ok(c) => c,
        Err(_) => {
            return CleanupResult {
                cleaned: false,
                deleted: false,
            }
        }
    };

    let mut start_idx = existing.find(SECTION_START);
    let mut end_idx = existing.find(SECTION_END);
    let mut section_end = SECTION_END;

    if start_idx.is_none() || end_idx.is_none() {
        start_idx = existing.find(LEGACY_SECTION_START);
        end_idx = existing.find(LEGACY_SECTION_END);
        section_end = LEGACY_SECTION_END;
        if start_idx.is_none() || end_idx.is_none() {
            return CleanupResult {
                cleaned: false,
                deleted: false,
            };
        }
    }

    let start = start_idx.unwrap();
    let end = end_idx.unwrap();
    let before = &existing[..start];
    let after = &existing[end + section_end.len()..];
    let combined = format!("{before}{after}");

    // Replace 3+ newlines with \n\n (like TS `/\n{3,}/g`)
    let mut remaining = String::new();
    let mut consec = 0usize;
    for ch in combined.chars() {
        if ch == '\n' {
            consec += 1;
            if consec <= 2 {
                remaining.push(ch);
            }
            // skip third+ consecutive
            if consec == 3 {
                // we already pushed 2, so third is skipped; further also skipped
                // actually we need to keep at most 2
                // So if consec >2, don't push
                // We pushed when consec<=2, so for 3 we already not pushing (since we check <=2)
                // But we pushed for 1 and 2, so need to handle correctly:
                // The above logic already skips when consec>2 because we only push if <=2
            }
        } else {
            consec = 0;
            remaining.push(ch);
        }
    }
    // The above already implements max 2 newlines; but need to ensure we didn't miss: we pushed for consec 1,2, skipped 3+
    // For correctness, the loop above already does that, but we had an off: we checked consec before increment? Let's re-implement simpler:
    // Actually we incremented then checked <=2, so it is correct.

    let trimmed = remaining.trim().to_string();

    if trimmed.is_empty() || trimmed == "# CLAUDE.md" {
        let _ = fs::remove_file(&output_path);
        return CleanupResult {
            cleaned: true,
            deleted: true,
        };
    }

    let _ = fs::write(&output_path, format!("{trimmed}\n"));
    CleanupResult {
        cleaned: true,
        deleted: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn no_file_returns_not_cleaned() {
        let dir = tempdir().unwrap();
        let r = cleanup_claude_md(dir.path());
        assert_eq!(
            r,
            CleanupResult {
                cleaned: false,
                deleted: false
            }
        );
    }

    #[test]
    fn strips_section_and_keeps_rest() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        let content =
            format!("# CLAUDE.md\nHello\n{SECTION_START}\nskill content\n{SECTION_END}\nWorld\n");
        fs::write(&path, content).unwrap();
        let r = cleanup_claude_md(dir.path());
        assert_eq!(r.cleaned, true);
        assert_eq!(r.deleted, false);
        let remaining = fs::read_to_string(&path).unwrap();
        assert!(remaining.contains("Hello"));
        assert!(remaining.contains("World"));
        assert!(!remaining.contains("skill content"));
        assert!(!remaining.contains(SECTION_START));
    }

    #[test]
    fn deletes_if_only_header() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        let content = format!("# CLAUDE.md\n{SECTION_START}\nfoo\n{SECTION_END}\n");
        fs::write(&path, content).unwrap();
        let r = cleanup_claude_md(dir.path());
        assert_eq!(r.cleaned, true);
        assert_eq!(r.deleted, true);
        assert!(!path.exists());
    }

    #[test]
    fn deletes_if_empty_after_strip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        let content = format!("{SECTION_START}\nfoo\n{SECTION_END}\n");
        fs::write(&path, content).unwrap();
        let r = cleanup_claude_md(dir.path());
        assert_eq!(r.deleted, true);
        assert!(!path.exists());
    }

    #[test]
    fn legacy_markers_also_stripped() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        let content =
            format!("keep\n{LEGACY_SECTION_START}\nlegacy\n{LEGACY_SECTION_END}\nkeep2\n");
        fs::write(&path, content).unwrap();
        let r = cleanup_claude_md(dir.path());
        assert!(r.cleaned);
        let remaining = fs::read_to_string(&path).unwrap();
        assert!(remaining.contains("keep"));
        assert!(!remaining.contains("legacy"));
    }

    #[test]
    fn no_markers_returns_not_cleaned() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        fs::write(&path, "# CLAUDE.md\nSome content\n").unwrap();
        let r = cleanup_claude_md(dir.path());
        assert_eq!(r.cleaned, false);
        assert_eq!(r.deleted, false);
        // file unchanged
        assert!(path.exists());
    }

    #[test]
    fn collapses_triple_newlines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        let content = format!("a\n\n\n\n{SECTION_START}\nx\n{SECTION_END}\n\n\n\nb\n");
        fs::write(&path, content).unwrap();
        cleanup_claude_md(dir.path());
        let remaining = fs::read_to_string(&path).unwrap();
        // Should not contain 3 consecutive newlines
        assert!(!remaining.contains("\n\n\n"));
    }
}
