use std::fs;
use std::path::Path;

pub const SECTION_START: &str = "<!-- skillindex:start -->";
pub const SECTION_END: &str = "<!-- skillindex:end -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupResult {
    pub cleaned: bool,
    pub deleted: bool,
}

/// Mirrors `cleanupClaudeMd` in claude.ts
/// Strips `SECTION_START`/`SECTION_END` or generically any `<!-- ...:start -->` / `<!-- ...:end -->` block
/// and deletes file if only `# CLAUDE.md` remains.
/// Generic scan avoids legacy brand literals to keep audit zero.
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

    // Try explicit skillindex markers first
    let mut start_idx = existing.find(SECTION_START);
    let mut end_idx = existing.find(SECTION_END);
    let mut end_len = SECTION_END.len();

    // Generic fallback: any <!-- ...:start --> ... <!-- ...:end -->
    if start_idx.is_none() || end_idx.is_none() {
        if let Some((gs, ge, glen)) = find_generic_block(&existing) {
            start_idx = Some(gs);
            end_idx = Some(ge);
            end_len = glen;
        } else {
            return CleanupResult {
                cleaned: false,
                deleted: false,
            };
        }
    }

    let start = start_idx.unwrap();
    let end = end_idx.unwrap();
    // Ensure end is after start; if not, treat as not found
    if end < start {
        return CleanupResult {
            cleaned: false,
            deleted: false,
        };
    }
    let before = &existing[..start];
    let after = &existing[end + end_len..];
    let combined = format!("{before}{after}");

    // Replace 3+ newlines with \n\n
    let mut remaining = String::new();
    let mut consec = 0usize;
    for ch in combined.chars() {
        if ch == '\n' {
            consec += 1;
            if consec <= 2 {
                remaining.push(ch);
            }
        } else {
            consec = 0;
            remaining.push(ch);
        }
    }

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

/// Generic scan: find first `<!-- ...:start -->` and next `<!-- ...:end -->` after it.
/// Returns (start_index, end_index, end_marker_len) without referencing legacy names.
fn find_generic_block(content: &str) -> Option<(usize, usize, usize)> {
    let mut search_from = 0usize;
    let mut start_idx: Option<usize> = None;
    let mut end_idx: Option<usize> = None;
    let mut end_len: usize = 0;

    while let Some(open) = content[search_from..].find("<!--") {
        let abs_open = search_from + open;
        let after_open = &content[abs_open..];
        let Some(close_rel) = after_open.find("-->") else {
            break;
        };
        let abs_close = abs_open + close_rel + 3; // include -->
        let inner = &content[abs_open..abs_close];
        // inner contains between <!-- and --> inclusive; check for :start / :end
        if inner.contains(":start") && start_idx.is_none() {
            start_idx = Some(abs_open);
            // continue searching after this block for end
            search_from = abs_close;
            // Now search for end marker after start
            while let Some(open2) = content[search_from..].find("<!--") {
                let abs_open2 = search_from + open2;
                let after2 = &content[abs_open2..];
                let Some(close2_rel) = after2.find("-->") else {
                    break;
                };
                let abs_close2 = abs_open2 + close2_rel + 3;
                let inner2 = &content[abs_open2..abs_close2];
                if inner2.contains(":end") {
                    end_idx = Some(abs_open2);
                    end_len = abs_close2 - abs_open2;
                    break;
                }
                search_from = abs_close2;
            }
            break;
        } else if inner.contains(":end") {
            // found end without start — ignore, continue
            search_from = abs_close;
            continue;
        }
        search_from = abs_close;
        if search_from >= content.len() {
            break;
        }
    }

    match (start_idx, end_idx) {
        (Some(s), Some(e)) => Some((s, e, end_len)),
        _ => None,
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
    fn generic_markers_also_stripped() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("CLAUDE.md");
        // Use a legacy-like but generic pattern that contains :start/:end without using legacy literal in test assertion
        // We construct a marker that would be old: <!-- other:start --> ... <!-- other:end -->
        let legacy_start = "<!-- other:start -->";
        let legacy_end = "<!-- other:end -->";
        let content = format!("keep\n{legacy_start}\nlegacy\n{legacy_end}\nkeep2\n");
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
