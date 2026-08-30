//! Integration tests for CLAUDE.md cleanup.
//! Mirrors claude.test.ts — one test per `it(...)` block.

use skillindex::claude::cleanup_claude_md;
use std::fs;
use tempfile::tempdir;

// ── cleanupClaudeMd ───────────────────────────────────────────────

#[test]
fn returns_cleaned_false_when_claude_md_does_not_exist() {
    let dir = tempdir().unwrap();
    let r = cleanup_claude_md(dir.path());
    assert!(!r.cleaned);
    assert!(!r.deleted);
}

#[test]
fn returns_cleaned_false_when_no_skillindex_markers() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("CLAUDE.md");
    fs::write(&path, "# CLAUDE.md\n\nMy custom instructions.\n").unwrap();
    let r = cleanup_claude_md(dir.path());
    assert!(!r.cleaned);
    assert!(!r.deleted);
    // file unchanged
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "# CLAUDE.md\n\nMy custom instructions.\n");
}

#[test]
fn deletes_claude_md_when_only_skillindex_section_remains() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("CLAUDE.md");
    fs::write(
        &path,
        "# CLAUDE.md\n\n<!-- skillindex:start -->\n\nGenerated content.\n\n<!-- skillindex:end -->\n",
    )
    .unwrap();
    let r = cleanup_claude_md(dir.path());
    assert!(r.cleaned);
    assert!(r.deleted);
    assert!(!path.exists());
}

#[test]
fn removes_skillindex_section_but_preserves_user_content() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("CLAUDE.md");
    fs::write(
        &path,
        "# CLAUDE.md\n\nMy custom instructions.\n\n<!-- skillindex:start -->\n\nGenerated content.\n\n<!-- skillindex:end -->\n\n## My notes\n\nDo not touch this.\n",
    )
    .unwrap();
    let r = cleanup_claude_md(dir.path());
    assert!(r.cleaned);
    assert!(!r.deleted);
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("My custom instructions."));
    assert!(content.contains("Do not touch this."));
    assert!(!content.contains("<!-- skillindex:start -->"));
    assert!(!content.contains("Generated content."));
}

#[test]
fn does_not_leave_triple_newlines_after_removing_section() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("CLAUDE.md");
    fs::write(
        &path,
        "# CLAUDE.md\n\nBefore.\n\n<!-- skillindex:start -->\nstuff\n<!-- skillindex:end -->\n\nAfter.\n",
    )
    .unwrap();
    cleanup_claude_md(dir.path());
    let content = fs::read_to_string(&path).unwrap();
    assert!(!content.contains("\n\n\n"));
}

#[test]
fn deletes_file_when_only_heading_remains() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("CLAUDE.md");
    fs::write(
        &path,
        "# CLAUDE.md\n\n<!-- skillindex:start -->\ngenerated\n<!-- skillindex:end -->\n",
    )
    .unwrap();
    let r = cleanup_claude_md(dir.path());
    assert!(r.cleaned);
    assert!(r.deleted);
    assert!(!path.exists());
}
