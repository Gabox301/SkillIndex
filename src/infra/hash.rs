use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

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
