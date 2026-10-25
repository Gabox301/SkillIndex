use sha2::{Digest, Sha256};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

/// Compute SHA-256 hex lowercase of a byte buffer — mirrors `sha256Buffer` in installer.ts
pub fn sha256_buffer(buf: &[u8]) -> String {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(buf);
    let result = hasher.finalize();
    result.iter().map(|b: &u8| format!("{b:02x}")).collect()
}

/// Compute SHA-256 hex lowercase of a file's contents — mirrors `sha256File` in installer.ts
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let data: Vec<u8> = fs::read(path)?;
    Ok(sha256_buffer(&data))
}

/// Normalize a registry relative path: `\` → `/` — mirrors `normalizeRegistryRelPath`
pub fn normalize_registry_rel_path(rel: &str) -> String {
    rel.replace('\\', "/")
}

/// Why a registry relative path was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryRelPathProblem {
    /// The rel is empty.
    Empty,
    /// The rel contains a NUL byte.
    NulByte,
    /// The rel is absolute (`/x`, `//host/share`, `C:/x`, `C:x`).
    Absolute,
    /// A segment is empty (`a//b`, `a/`), i.e. the rel is not canonical.
    EmptySegment,
    /// A segment is `.`, which is redundant and hides intent.
    DotSegment,
    /// A segment is `..`, i.e. the rel escapes its base directory.
    ParentSegment,
}

impl fmt::Display for RegistryRelPathProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg: &str = match self {
            RegistryRelPathProblem::Empty => "la ruta está vacía",
            RegistryRelPathProblem::NulByte => "la ruta contiene un byte NUL",
            RegistryRelPathProblem::Absolute => "no se permiten rutas absolutas",
            RegistryRelPathProblem::EmptySegment => "la ruta contiene un segmento vacío",
            RegistryRelPathProblem::DotSegment => "la ruta contiene un segmento '.'",
            RegistryRelPathProblem::ParentSegment => "la ruta contiene un segmento '..'",
        };
        f.write_str(msg)
    }
}

/// A registry relative path that failed [`validate_registry_rel_path`].
///
/// It always carries the offending rel verbatim, so every caller can fail
/// closed with a message that names the exact path the registry asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidRegistryRelPath {
    rel: String,
    problem: RegistryRelPathProblem,
}

impl InvalidRegistryRelPath {
    /// The offending rel exactly as it appeared in the registry.
    pub fn rel(&self) -> &str {
        &self.rel
    }

    /// The rule the rel violated.
    pub fn problem(&self) -> RegistryRelPathProblem {
        self.problem
    }
}

impl fmt::Display for InvalidRegistryRelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ruta relativa del registro rechazada {:?}: {}",
            self.rel, self.problem
        )
    }
}

impl std::error::Error for InvalidRegistryRelPath {}

/// Validate and normalize a registry relative path.
///
/// This is the single gate for every place that joins a registry `rel` onto a
/// directory (verify, download write, cache copy, local copy). It normalizes
/// backslashes to `/` first, so Windows-style traversal (`..` + `\` + `..`) is
/// rejected with exactly the same rules as `../../x`, and it never resolves a
/// path: anything that is not a plain relative chain of real segments is
/// rejected.
///
/// On success it returns the normalized rel (backslashes to `/`).
pub fn validate_registry_rel_path(rel: &str) -> Result<String, InvalidRegistryRelPath> {
    fn reject(
        rel: &str,
        problem: RegistryRelPathProblem,
    ) -> Result<String, InvalidRegistryRelPath> {
        Err(InvalidRegistryRelPath {
            rel: rel.to_string(),
            problem,
        })
    }

    if rel.is_empty() {
        return reject(rel, RegistryRelPathProblem::Empty);
    }
    if rel.contains('\0') {
        return reject(rel, RegistryRelPathProblem::NulByte);
    }
    let normalized: String = normalize_registry_rel_path(rel);
    // After normalization a leading `/` covers POSIX absolutes and Windows UNC
    // share paths (double backslash + host + share), which become `//host/share`.
    if normalized.starts_with('/') || has_drive_prefix(&normalized) {
        return reject(rel, RegistryRelPathProblem::Absolute);
    }
    for segment in normalized.split('/') {
        match segment {
            "" => return reject(rel, RegistryRelPathProblem::EmptySegment),
            "." => return reject(rel, RegistryRelPathProblem::DotSegment),
            ".." => return reject(rel, RegistryRelPathProblem::ParentSegment),
            _ => {}
        }
    }
    Ok(normalized)
}

/// True for Windows drive prefixes (`C:`, `C:relative`) once the rel is normalized.
fn has_drive_prefix(normalized: &str) -> bool {
    let bytes: &[u8] = normalized.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

/// Join a validated registry `rel` onto `base`.
///
/// The join is traversal-proof because it only ever appends plain, non-empty
/// segments: validation rejects absolutes, `..`, `.`, empty segments, NUL bytes
/// and backslash tricks before a single component is pushed.
pub fn join_registry_rel(base: &Path, rel: &str) -> Result<PathBuf, InvalidRegistryRelPath> {
    let normalized: String = validate_registry_rel_path(rel)?;
    let mut out: PathBuf = base.to_path_buf();
    for segment in normalized.split('/') {
        out.push(segment);
    }
    Ok(out)
}

/// Returns true if the relative path is a disallowed archive (`.zip` case-insensitive)
pub fn is_disallowed_skill_file(rel: &str) -> bool {
    normalize_registry_rel_path(rel)
        .to_lowercase()
        .ends_with(".zip")
}

/// Normalize CRLF/CR line endings to LF for text files.
///
/// Binary files are returned untouched: formats like PNG/JPG legitimately
/// contain `0x0D` bytes (e.g. the PNG magic ends in `0D 0A`) and replacing
/// them corrupts the file (broken CRCs, unrenderable images). Detection is
/// UTF-8 validity — text (including CRLF text) is valid UTF-8, while
/// true binaries almost never are.
pub fn normalize_line_endings(data: &[u8]) -> Vec<u8> {
    if std::str::from_utf8(data).is_err() {
        return data.to_vec();
    }
    let s: std::borrow::Cow<'_, str> = String::from_utf8_lossy(data);
    if !s.contains('\r') {
        return data.to_vec();
    }
    s.replace("\r\n", "\n").replace('\r', "\n").into_bytes()
}

/// Compute bundle hash: `sha256(sorted "rel:sha256" join "\n")` hex lower
/// `entries` is slice of `(rel, hex_sha256)`. Rel paths are normalized `\`→`/` and sorted.
pub fn bundle_hash(entries: &[(String, String)]) -> String {
    let mut normalized: Vec<(String, String)> = entries
        .iter()
        .map(|(rel, hash)| (normalize_registry_rel_path(rel), hash.clone()))
        .collect();
    normalized.sort_by(|a: &(String, String), b: &(String, String)| a.0.cmp(&b.0));
    let joined: String = normalized
        .iter()
        .map(|(rel, hash)| format!("{rel}:{hash}"))
        .collect::<Vec<_>>()
        .join("\n");
    sha256_buffer(joined.as_bytes())
}

/// Convenience: compute bundle hash directly from file buffers (rel + raw bytes)
pub fn bundle_hash_from_buffers(entries: &[(String, Vec<u8>)]) -> String {
    let hashed: Vec<(String, String)> = entries
        .iter()
        .map(|(rel, buf)| (rel.clone(), sha256_buffer(buf)))
        .collect();
    bundle_hash(&hashed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn sha256_buffer_known_vectors() {
        // empty
        assert_eq!(
            sha256_buffer(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // hello
        assert_eq!(
            sha256_buffer(b"hello"),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        // ensure lower hex
        let h: String = sha256_buffer(b"test");
        assert_eq!(h, h.to_lowercase());
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn sha256_file_matches_buffer() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let p: std::path::PathBuf = dir.path().join("file.txt");
        fs::write(&p, b"hello world").unwrap();
        let file_hash: String = sha256_file(&p).unwrap();
        let buf_hash: String = sha256_buffer(b"hello world");
        assert_eq!(file_hash, buf_hash);
    }

    #[test]
    fn sha256_file_missing_returns_error() {
        let result: Result<String, std::io::Error> =
            sha256_file(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn normalize_line_endings_crlf_text() {
        assert_eq!(normalize_line_endings(b"a\r\nb\r\n"), b"a\nb\n");
        assert_eq!(normalize_line_endings(b"a\rb"), b"a\nb");
        assert_eq!(normalize_line_endings(b"a\nb\n"), b"a\nb\n");
    }

    #[test]
    fn normalize_line_endings_leaves_binaries_untouched() {
        // PNG magic ends in 0D 0A; mangling it corrupts CRCs and images.
        let png_magic: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(normalize_line_endings(png_magic), png_magic);
        let binary: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0x0D, 0x0A, 0x00, 0x80];
        assert_eq!(normalize_line_endings(&binary), binary);
    }

    #[test]
    fn normalize_registry_rel_path_backslash() {
        assert_eq!(normalize_registry_rel_path("a\\b\\c.md"), "a/b/c.md");
        assert_eq!(
            normalize_registry_rel_path("references\\notes.md"),
            "references/notes.md"
        );
        assert_eq!(normalize_registry_rel_path("SKILL.md"), "SKILL.md");
    }

    #[test]
    fn validate_registry_rel_path_rejects_parent_segment() {
        let err: InvalidRegistryRelPath =
            validate_registry_rel_path("../../etc/passwd").unwrap_err();
        assert_eq!(err.problem(), RegistryRelPathProblem::ParentSegment);
        assert_eq!(err.rel(), "../../etc/passwd");
        assert!(
            err.to_string().contains("../../etc/passwd"),
            "error must name the offending rel: {err}"
        );
        // traversal hidden behind a valid-looking prefix
        assert_eq!(
            validate_registry_rel_path("references/../SKILL.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::ParentSegment
        );
        assert_eq!(
            validate_registry_rel_path("..").unwrap_err().problem(),
            RegistryRelPathProblem::ParentSegment
        );
    }

    #[test]
    fn validate_registry_rel_path_rejects_absolute() {
        for rel in [
            "/etc/passwd",
            "\\\\host\\share\\evil.md",
            "C:/Windows/evil.md",
            "c:relative\\evil.md",
        ] {
            let err: InvalidRegistryRelPath = validate_registry_rel_path(rel).unwrap_err();
            assert_eq!(
                err.problem(),
                RegistryRelPathProblem::Absolute,
                "rel {rel:?} should be rejected as absolute"
            );
            assert_eq!(err.rel(), rel);
        }
    }

    #[test]
    fn validate_registry_rel_path_rejects_empty_and_dot_segments() {
        assert_eq!(
            validate_registry_rel_path("").unwrap_err().problem(),
            RegistryRelPathProblem::Empty
        );
        assert_eq!(
            validate_registry_rel_path("references//notes.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::EmptySegment
        );
        assert_eq!(
            validate_registry_rel_path("references/")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::EmptySegment
        );
        assert_eq!(
            validate_registry_rel_path("./SKILL.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::DotSegment
        );
        assert_eq!(
            validate_registry_rel_path("references/./notes.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::DotSegment
        );
    }

    #[test]
    fn validate_registry_rel_path_rejects_backslash_traversal() {
        // Normalization runs first, so Windows-style traversal is caught too.
        let err: InvalidRegistryRelPath =
            validate_registry_rel_path("..\\..\\windows\\system32\\evil.md").unwrap_err();
        assert_eq!(err.problem(), RegistryRelPathProblem::ParentSegment);
        assert_eq!(err.rel(), "..\\..\\windows\\system32\\evil.md");
        assert_eq!(
            validate_registry_rel_path("references\\..\\..\\evil.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::ParentSegment
        );
        assert_eq!(
            validate_registry_rel_path("references\\..\\notes.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::ParentSegment
        );
        assert_eq!(
            validate_registry_rel_path(".\\SKILL.md")
                .unwrap_err()
                .problem(),
            RegistryRelPathProblem::DotSegment
        );
    }

    #[test]
    fn validate_registry_rel_path_rejects_nul_byte() {
        let err: InvalidRegistryRelPath = validate_registry_rel_path("SKILL.md\0.md").unwrap_err();
        assert_eq!(err.problem(), RegistryRelPathProblem::NulByte);
        assert!(err.rel().contains("SKILL.md"));
    }

    #[test]
    fn validate_registry_rel_path_accepts_nested_references() {
        assert_eq!(
            validate_registry_rel_path("references/notes.md").unwrap(),
            "references/notes.md"
        );
        // backslash normalization is preserved for legit paths
        assert_eq!(
            validate_registry_rel_path("references\\notes.md").unwrap(),
            "references/notes.md"
        );
        assert_eq!(validate_registry_rel_path("SKILL.md").unwrap(), "SKILL.md");
    }

    #[test]
    fn join_registry_rel_happy_path_nested_references() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let base: std::path::PathBuf = dir.path().join("my-skill");
        fs::create_dir_all(&base).unwrap();
        let dest: std::path::PathBuf = join_registry_rel(&base, "references\\notes.md").unwrap();
        assert_eq!(dest, base.join("references").join("notes.md"));
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::write(&dest, b"notes").unwrap();
        assert_eq!(fs::read_to_string(&dest).unwrap(), "notes");
        assert_eq!(
            join_registry_rel(&base, "SKILL.md").unwrap(),
            base.join("SKILL.md")
        );
    }

    #[test]
    fn join_registry_rel_never_escapes_base() {
        let dir: tempfile::TempDir = tempdir().unwrap();
        let base: std::path::PathBuf = dir.path().join("my-skill");
        fs::create_dir_all(&base).unwrap();
        for rel in [
            "../escape.md",
            "..\\escape.md",
            "/tmp/escape.md",
            "references/../escape.md",
        ] {
            let joined: Result<std::path::PathBuf, InvalidRegistryRelPath> =
                join_registry_rel(&base, rel);
            assert!(joined.is_err(), "rel {rel:?} must not join");
        }
        assert!(!dir.path().join("escape.md").exists());
    }

    #[test]
    fn is_disallowed_skill_file_zip() {
        assert!(is_disallowed_skill_file("archive.zip"));
        assert!(is_disallowed_skill_file("downloads/tool.ZIP"));
        assert!(is_disallowed_skill_file("a\\b\\c.Zip"));
        assert!(!is_disallowed_skill_file("SKILL.md"));
        assert!(!is_disallowed_skill_file("archive.zipx"));
        assert!(!is_disallowed_skill_file("zip.txt"));
    }

    #[test]
    fn bundle_hash_single_entry() {
        let entries: Vec<(String, String)> = vec![(
            "SKILL.md".to_string(),
            "8e1a9758b9721b48f534cee5998a3fcea8833f4885ca5929847f9fc48b773957".to_string(),
        )];
        let joined = "SKILL.md:8e1a9758b9721b48f534cee5998a3fcea8833f4885ca5929847f9fc48b773957";
        let expected: String = sha256_buffer(joined.as_bytes());
        assert_eq!(bundle_hash(&entries), expected);
        // matches real fixture: bun
        assert_eq!(
            bundle_hash(&entries),
            "1386ed5490ed278e38312a416c9aa074dbe3849f32ae7f841e4142fef5d4df94"
        );
    }

    #[test]
    fn bundle_hash_sorted() {
        // a.md:h1,b.md:h2 sorted -> sha256("a.md:h1\nb.md:h2")
        let entries_a: Vec<(String, String)> = vec![
            ("b.md".to_string(), "h2".to_string()),
            ("a.md".to_string(), "h1".to_string()),
        ];
        let entries_b: Vec<(String, String)> = vec![
            ("a.md".to_string(), "h1".to_string()),
            ("b.md".to_string(), "h2".to_string()),
        ];
        assert_eq!(bundle_hash(&entries_a), bundle_hash(&entries_b));
        let expected: String = sha256_buffer(b"a.md:h1\nb.md:h2");
        assert_eq!(bundle_hash(&entries_a), expected);
    }

    #[test]
    fn bundle_hash_backslash_normalized() {
        let entries_bs: Vec<(String, String)> = vec![
            ("references\\notes.md".to_string(), "abc".to_string()),
            ("SKILL.md".to_string(), "def".to_string()),
        ];
        let entries_slash: Vec<(String, String)> = vec![
            ("references/notes.md".to_string(), "abc".to_string()),
            ("SKILL.md".to_string(), "def".to_string()),
        ];
        assert_eq!(bundle_hash(&entries_bs), bundle_hash(&entries_slash));
    }

    #[test]
    fn bundle_hash_from_buffers_matches_hash() {
        let buffers: Vec<(String, Vec<u8>)> = vec![
            ("a.md".to_string(), b"content a".to_vec()),
            ("b.md".to_string(), b"content b".to_vec()),
        ];
        let hashed: Vec<(String, String)> = buffers
            .iter()
            .map(|(rel, buf)| (rel.clone(), sha256_buffer(buf)))
            .collect();
        assert_eq!(bundle_hash_from_buffers(&buffers), bundle_hash(&hashed));
    }

    #[test]
    fn bundle_hash_219_fixtures_parity() {
        // Load real registry and verify bundle hashes for all entries match
        // Skips placeholder entries like `elysiajs` where bundleHash is not a hex digest (known bad fixture).
        let manifest_path: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("skills-registry")
            .join("index.json");
        let manifest_path = if manifest_path.exists() {
            manifest_path
        } else {
            // fallback: try parent
            let alt: std::path::PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("skills-registry")
                .join("index.json");
            if alt.exists() {
                alt
            } else {
                return;
            }
        };
        let data: String = fs::read_to_string(&manifest_path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        let skills: &serde_json::Map<String, serde_json::Value> =
            v.get("skills").unwrap().as_object().unwrap();
        assert!(
            skills.len() >= 100,
            "expected at least 100 fixtures, got {}",
            skills.len()
        );
        let mut verified: usize = 0usize;
        for (name, entry) in skills {
            let files: &Vec<serde_json::Value> = entry.get("files").unwrap().as_array().unwrap();
            let sha_map: &serde_json::Map<String, serde_json::Value> =
                entry.get("sha256").unwrap().as_object().unwrap();
            let expected_bundle: &str = entry.get("bundleHash").unwrap().as_str().unwrap();
            // Skip placeholder non-hex bundle hashes (e.g. "elysiajs-skill-hash")
            if expected_bundle.len() != 64
                || !expected_bundle.chars().all(|c: char| c.is_ascii_hexdigit())
            {
                continue;
            }
            let entries: Vec<(String, String)> = files
                .iter()
                .map(|f: &serde_json::Value| {
                    let rel: String = f.as_str().unwrap().to_string();
                    let hash: String = sha_map.get(&rel).unwrap().as_str().unwrap().to_string();
                    (rel, hash)
                })
                .collect();
            let computed: String = bundle_hash(&entries);
            assert_eq!(
                computed, expected_bundle,
                "bundle hash mismatch for skill {name}"
            );
            verified += 1;
        }
        assert!(
            verified >= 217,
            "expected at least 217 verified fixtures, got {verified}"
        );
    }
}
