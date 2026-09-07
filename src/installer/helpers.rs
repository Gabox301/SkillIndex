use std::env;
use std::fs;
use std::path::Path;

use crate::infra::hash::normalize_registry_rel_path;

fn get_github_token() -> Option<String> {
    if let Ok(v) = env::var("GITHUB_TOKEN")
        && !v.trim().is_empty()
    {
        return Some(v);
    }
    if let Ok(v) = env::var("GH_TOKEN")
        && !v.trim().is_empty()
    {
        return Some(v);
    }
    None
}

fn is_githubusercontent_url(url: &str) -> bool {
    // extract host between "://" and next "/"
    let host: &str = if let Some(start) = url.find("://") {
        let rest: &str = &url[start + 3..];
        let end: usize = rest.find('/').unwrap_or(rest.len());
        &rest[..end]
    } else {
        url
    };
    let lower: String = host.to_ascii_lowercase();
    lower == "raw.githubusercontent.com"
        || lower.ends_with(".githubusercontent.com")
        || lower == "githubusercontent.com"
}

pub fn github_download_headers(url: &str) -> reqwest::header::HeaderMap {
    let mut headers: reqwest::header::HeaderMap = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("skillindex"),
    );
    if let Some(token) = get_github_token()
        && is_githubusercontent_url(url)
    {
        let bearer: String = format!("Bearer {token}");
        if let Ok(v) = reqwest::header::HeaderValue::from_str(&bearer) {
            headers.insert(reqwest::header::AUTHORIZATION, v);
        }
    }
    headers
}

fn encode_uri_component(s: &str) -> String {
    let mut out: String = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        let c: char = b as char;
        if matches!(
            c,
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')'
        ) {
            out.push(c);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

pub fn encode_raw_path(skill_name: &str, rel: &str) -> String {
    let normalized: String = normalize_registry_rel_path(rel);
    let mut segments: Vec<String> = Vec::new();
    segments.push(encode_uri_component(skill_name));
    for part in normalized.split('/') {
        segments.push(encode_uri_component(part));
    }
    segments.join("/")
}

pub fn rel_path_from_to(from: &Path, to: &Path) -> String {
    let from_comps: Vec<_> = from.components().collect();
    let to_comps: Vec<_> = to.components().collect();
    let mut common: usize = 0usize;
    for (a, b) in from_comps.iter().zip(to_comps.iter()) {
        if a == b {
            common += 1;
        } else {
            break;
        }
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in common..from_comps.len() {
        parts.push("..".to_string());
    }
    for comp in to_comps.iter().skip(common) {
        parts.push(comp.as_os_str().to_string_lossy().to_string());
    }
    if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/").replace('\\', "/")
    }
}

pub fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry: fs::DirEntry = entry?;
        let ty: fs::FileType = entry.file_type()?;
        let s: std::path::PathBuf = entry.path();
        let d: std::path::PathBuf = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&s, &d)?;
        } else if ty.is_file() {
            fs::copy(&s, &d)?;
        }
    }
    Ok(())
}

pub fn ensure_symlink_to(target: &Path, link_path: &Path) -> std::io::Result<()> {
    if let Some(parent) = link_path.parent() {
        fs::create_dir_all(parent)?;
    }
    // remove existing
    if link_path.exists() || link_path.is_symlink() {
        let _ = fs::remove_dir_all(link_path);
        let _ = fs::remove_file(link_path);
        // try both
        if link_path.exists() {
            let _ = fs::remove_dir_all(link_path);
        }
    }
    let rel: String = rel_path_from_to(link_path.parent().unwrap_or(Path::new(".")), target);
    #[cfg(windows)]
    {
        let res: Result<(), std::io::Error> = std::os::windows::fs::symlink_dir(&rel, link_path);
        match res {
            Ok(()) => Ok(()),
            Err(_) => {
                // fallback to copyDir on any error (mirrors TS catch-all)
                copy_dir(target, link_path)
            }
        }
    }
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(&rel, link_path) {
            Ok(()) => Ok(()),
            Err(_) => copy_dir(target, link_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn encode_raw_path_simple() {
        assert_eq!(encode_raw_path("my-skill", "SKILL.md"), "my-skill/SKILL.md");
        assert_eq!(
            encode_raw_path("my-skill", "references/notes.md"),
            "my-skill/references/notes.md"
        );
    }

    #[test]
    fn encode_raw_path_backslash() {
        assert_eq!(
            encode_raw_path("my-skill", "references\\notes.md"),
            "my-skill/references/notes.md"
        );
    }

    #[test]
    fn encode_raw_path_special_chars() {
        assert_eq!(
            encode_raw_path("my-skill", "file with spaces.md"),
            "my-skill/file%20with%20spaces.md"
        );
    }

    #[test]
    fn rel_path_from_to_basic() {
        let from: &Path = Path::new("/a/b/c");
        let to: &Path = Path::new("/a/b/d/e");
        assert_eq!(rel_path_from_to(from, to), "../d/e");
        assert_eq!(
            rel_path_from_to(Path::new("/a/b"), Path::new("/a/b/c/d")),
            "c/d"
        );
        assert_eq!(
            rel_path_from_to(Path::new("/a/b/c"), Path::new("/a/b/c")),
            "."
        );
    }

    #[test]
    fn copy_dir_recursively() {
        let src: tempfile::TempDir = tempdir().unwrap();
        let dest: tempfile::TempDir = tempdir().unwrap();
        fs::create_dir_all(src.path().join("sub")).unwrap();
        fs::write(src.path().join("a.txt"), b"hello").unwrap();
        fs::write(src.path().join("sub/b.txt"), b"world").unwrap();
        let out: std::path::PathBuf = dest.path().join("out");
        copy_dir(src.path(), &out).unwrap();
        assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "hello");
        assert_eq!(fs::read_to_string(out.join("sub/b.txt")).unwrap(), "world");
    }

    #[test]
    fn ensure_symlink_to_creates_link_or_copy() {
        let tmp: tempfile::TempDir = tempdir().unwrap();
        let target: std::path::PathBuf = tmp.path().join("target");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("file.txt"), b"data").unwrap();
        let link: std::path::PathBuf = tmp.path().join("link").join("skill");
        ensure_symlink_to(&target, &link).unwrap();
        // either symlink or copy, file should be accessible
        assert!(link.exists() || link.is_symlink());
        assert_eq!(fs::read_to_string(link.join("file.txt")).unwrap(), "data");
    }

    #[test]
    fn is_githubusercontent_detection() {
        assert!(is_githubusercontent_url(
            "https://raw.githubusercontent.com/foo/bar"
        ));
        assert!(is_githubusercontent_url(
            "https://raw.githubusercontent.com/Gabox301/SkillIndex/main/file"
        ));
        assert!(!is_githubusercontent_url(
            "https://example.test/skills-registry/file"
        ));
        assert!(!is_githubusercontent_url("https://github.com/foo"));
    }

    #[test]
    fn github_headers_include_bearer_on_github() {
        let _env_guard: crate::infra::cache::EnvGuard = crate::infra::cache::env_lock();
        let prev: Option<String> = env::var("GITHUB_TOKEN").ok();
        unsafe { env::set_var("GITHUB_TOKEN", "test-token-123") };
        let headers: reqwest::header::HeaderMap = github_download_headers(
            "https://raw.githubusercontent.com/Gabox301/SkillIndex/main/file",
        );
        assert_eq!(
            headers.get("authorization").unwrap().to_str().unwrap(),
            "Bearer test-token-123"
        );
        let headers2: reqwest::header::HeaderMap =
            github_download_headers("https://example.test/file");
        assert!(headers2.get("authorization").is_none());
        match prev {
            Some(v) => unsafe { env::set_var("GITHUB_TOKEN", v) },
            None => unsafe { env::remove_var("GITHUB_TOKEN") },
        }
    }

    #[test]
    fn github_headers_no_token_no_auth() {
        let _env_guard: crate::infra::cache::EnvGuard = crate::infra::cache::env_lock();
        let prev: Option<String> = env::var("GITHUB_TOKEN").ok();
        let prev2: Option<String> = env::var("GH_TOKEN").ok();
        unsafe {
            env::remove_var("GITHUB_TOKEN");
            env::remove_var("GH_TOKEN")
        };
        let headers: reqwest::header::HeaderMap =
            github_download_headers("https://raw.githubusercontent.com/foo/bar");
        assert!(headers.get("authorization").is_none());
        if let Some(v) = prev {
            unsafe { env::set_var("GITHUB_TOKEN", v) };
        }
        if let Some(v) = prev2 {
            unsafe { env::set_var("GH_TOKEN", v) };
        }
    }

    #[test]
    fn helpers_module_imports_remain_valid() {
        // Guard against import regressions; exercises a public helper
        let _ = rel_path_from_to(Path::new("/a"), Path::new("/b"));
    }
}
