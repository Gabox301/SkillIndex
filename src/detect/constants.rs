use std::collections::HashSet;
use std::sync::LazyLock;

/// Directories that are never descended during scans — mirrors `SCAN_SKIP_DIRS` in lib.ts
pub static SCAN_SKIP_DIRS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "node_modules",
        ".git",
        "vendor",
        ".next",
        "dist",
        "build",
        ".output",
        ".nuxt",
        ".svelte-kit",
        "__pycache__",
        ".cache",
        "coverage",
        ".turbo",
        ".terraform",
        "var",
        "bin",
        "obj",
        ".vs",
    ])
});

/// Returns true if the directory name should be skipped
pub fn is_skip_dir(name: &str) -> bool {
    SCAN_SKIP_DIRS.contains(name)
}

pub static FRONTEND_PACKAGES_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| crate::skills::FRONTEND_PACKAGES.iter().copied().collect());

pub static FRONTEND_BONUS_SKILLS: &[&str] = crate::skills::FRONTEND_BONUS_SKILLS;

/// IDs de combos de seguridad que son opcionales (requieren opt-in vía checkbox)
pub const OPTIONAL_SECURITY_COMBO_IDS: &[&str] = &[
    "security-operations",
    "red-team",
    "cloud-security",
    "forensics-ir",
];

pub fn is_optional_security_combo(id: &str) -> bool {
    OPTIONAL_SECURITY_COMBO_IDS.contains(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_skip_dirs_contains_expected_entries() {
        let expected = [
            "node_modules",
            ".git",
            "vendor",
            ".next",
            "dist",
            "build",
            ".output",
            ".nuxt",
            ".svelte-kit",
            "__pycache__",
            ".cache",
            "coverage",
            ".turbo",
            ".terraform",
            "var",
            "bin",
            "obj",
            ".vs",
        ];
        assert_eq!(SCAN_SKIP_DIRS.len(), expected.len());
        for e in expected {
            assert!(SCAN_SKIP_DIRS.contains(e), "missing {e}");
        }
    }

    #[test]
    fn is_skip_dir_true_for_known() {
        assert!(is_skip_dir("node_modules"));
        assert!(is_skip_dir(".git"));
        assert!(is_skip_dir("dist"));
        assert!(is_skip_dir("bin"));
        assert!(is_skip_dir(".vs"));
    }

    #[test]
    fn is_skip_dir_false_for_regular() {
        assert!(!is_skip_dir("src"));
        assert!(!is_skip_dir("packages"));
        assert!(!is_skip_dir("my-app"));
        assert!(!is_skip_dir(""));
    }

    #[test]
    fn cache_identity_is_stable() {
        let a = SCAN_SKIP_DIRS.len();
        let b = SCAN_SKIP_DIRS.len();
        assert_eq!(a, b);
        assert_eq!(a, 18);
    }
}
