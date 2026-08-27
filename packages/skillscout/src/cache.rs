use std::env;
use std::fs;
use std::path::PathBuf;

/// Returns the SkillScout cache directory:
/// `SKILLSCOUT_CACHE_DIR` || `AUTOSKILLS_CACHE_DIR` || `~/.cache/skillscout/skills-registry`
pub fn get_skillscout_cache_dir() -> PathBuf {
    if let Ok(v) = env::var("SKILLSCOUT_CACHE_DIR") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    if let Ok(v) = env::var("AUTOSKILLS_CACHE_DIR") {
        if !v.trim().is_empty() {
            return PathBuf::from(v);
        }
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".cache")
        .join("skillscout")
        .join("skills-registry")
}

/// Backward-compat alias — mirrors `getAutoskillsCacheDir` in installer.ts
pub fn get_autoskills_cache_dir() -> PathBuf {
    get_skillscout_cache_dir()
}

/// Returns the cache registry dir for a given bundle hash
pub fn get_cache_registry_dir(bundle_hash: &str) -> PathBuf {
    get_skillscout_cache_dir().join(bundle_hash)
}

/// Returns the cache registry dir for a RegistryEntry-like bundle hash
pub fn get_cache_registry_dir_for_entry(bundle_hash: &str) -> PathBuf {
    get_cache_registry_dir(bundle_hash)
}

/// Clears the SkillScout cache — mirrors `clearSkillScoutCache` in installer.ts
/// Returns `(cache_dir, removed)` where `removed` indicates whether the dir existed.
pub fn clear_skillscout_cache() -> (PathBuf, bool) {
    let dir = get_skillscout_cache_dir();
    let existed = dir.exists();
    // force remove — ignore errors like TS `force:true`
    let _ = fs::remove_dir_all(&dir);
    (dir, existed)
}

/// Backward-compat alias
pub fn clear_autoskills_cache() -> (PathBuf, bool) {
    clear_skillscout_cache()
}

#[cfg(test)]
pub(crate) static ENV_LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
#[cfg(test)]
thread_local! { static ENV_LOCK_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(crate) struct EnvGuard {
    _guard: Option<std::sync::MutexGuard<'static, ()>>,
}
#[cfg(test)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        if self._guard.is_some() {
            ENV_LOCK_COUNT.with(|c| c.set(0));
        } else {
            let count = ENV_LOCK_COUNT.with(|c| c.get());
            if count > 0 {
                ENV_LOCK_COUNT.with(|c| c.set(count - 1));
            }
        }
    }
}
#[cfg(test)]
pub(crate) fn env_lock() -> EnvGuard {
    let count = ENV_LOCK_COUNT.with(|c| c.get());
    if count > 0 {
        ENV_LOCK_COUNT.with(|c| c.set(count + 1));
        EnvGuard { _guard: None }
    } else {
        let guard = ENV_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        ENV_LOCK_COUNT.with(|c| c.set(1));
        EnvGuard {
            _guard: Some(guard),
        }
    }
}

/// Test helper: temporarily set env var and restore (serialized via global lock, re-entrant)
#[cfg(test)]
pub(crate) fn with_env_var<F, R>(key: &str, value: Option<&str>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let count = ENV_LOCK_COUNT.with(|c| c.get());
    // Acquire lock only if not already holding
    let _guard_opt: Option<std::sync::MutexGuard<'static, ()>> = if count == 0 {
        let g = ENV_LOCK
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        ENV_LOCK_COUNT.with(|c| c.set(1));
        Some(g)
    } else {
        ENV_LOCK_COUNT.with(|c| c.set(count + 1));
        None
    };
    let prev = env::var(key).ok();
    match value {
        Some(v) => unsafe { env::set_var(key, v) },
        None => unsafe { env::remove_var(key) },
    }
    let result = f();
    match prev {
        Some(v) => unsafe { env::set_var(key, v) },
        None => unsafe { env::remove_var(key) },
    }
    let new_count = ENV_LOCK_COUNT.with(|c| c.get());
    if _guard_opt.is_some() {
        ENV_LOCK_COUNT.with(|c| c.set(0));
        // _guard_opt dropped here releasing mutex
    } else {
        ENV_LOCK_COUNT.with(|c| c.set(new_count - 1));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn cache_dir_uses_skillscout_env() {
        with_env_var("SKILLSCOUT_CACHE_DIR", Some("/tmp/custom-cache"), || {
            with_env_var("AUTOSKILLS_CACHE_DIR", Some("/tmp/other"), || {
                assert_eq!(
                    get_skillscout_cache_dir(),
                    PathBuf::from("/tmp/custom-cache")
                );
            });
        });
    }

    #[test]
    fn cache_dir_falls_back_to_autoskills() {
        with_env_var("SKILLSCOUT_CACHE_DIR", None, || {
            with_env_var("AUTOSKILLS_CACHE_DIR", Some("/tmp/autoskills"), || {
                assert_eq!(get_skillscout_cache_dir(), PathBuf::from("/tmp/autoskills"));
            });
        });
    }

    #[test]
    fn cache_dir_falls_back_to_home() {
        with_env_var("SKILLSCOUT_CACHE_DIR", None, || {
            with_env_var("AUTOSKILLS_CACHE_DIR", None, || {
                let dir = get_skillscout_cache_dir();
                let home = dirs::home_dir().unwrap();
                assert_eq!(
                    dir,
                    home.join(".cache")
                        .join("skillscout")
                        .join("skills-registry")
                );
            });
        });
    }

    #[test]
    fn cache_dir_skillscout_takes_precedence() {
        with_env_var("SKILLSCOUT_CACHE_DIR", Some("/tmp/a"), || {
            with_env_var("AUTOSKILLS_CACHE_DIR", Some("/tmp/b"), || {
                assert_eq!(get_skillscout_cache_dir(), PathBuf::from("/tmp/a"));
            });
        });
    }

    #[test]
    fn cache_dir_empty_string_falls_through() {
        with_env_var("SKILLSCOUT_CACHE_DIR", Some(""), || {
            with_env_var("AUTOSKILLS_CACHE_DIR", Some("/tmp/fallback"), || {
                assert_eq!(get_skillscout_cache_dir(), PathBuf::from("/tmp/fallback"));
            });
        });
    }

    #[test]
    fn get_cache_registry_dir_appends_hash() {
        with_env_var("SKILLSCOUT_CACHE_DIR", Some("/tmp/cache-root"), || {
            let dir = get_cache_registry_dir("abc123");
            assert_eq!(dir, PathBuf::from("/tmp/cache-root").join("abc123"));
        });
    }

    #[test]
    fn clear_skillscout_cache_removes_dir() {
        let tmp = tempdir().unwrap();
        let cache_root = tmp.path().join("my-cache");
        fs::create_dir_all(&cache_root).unwrap();
        fs::write(cache_root.join("file.txt"), b"data").unwrap();
        assert!(cache_root.exists());

        with_env_var(
            "SKILLSCOUT_CACHE_DIR",
            Some(cache_root.to_str().unwrap()),
            || {
                let (dir, removed) = clear_skillscout_cache();
                assert_eq!(dir, cache_root);
                assert!(removed);
                assert!(!cache_root.exists());
            },
        );
    }

    #[test]
    fn clear_skillscout_cache_nonexistent_returns_false() {
        let tmp = tempdir().unwrap();
        let cache_root = tmp.path().join("nonexistent-cache-dir");
        assert!(!cache_root.exists());
        with_env_var(
            "SKILLSCOUT_CACHE_DIR",
            Some(cache_root.to_str().unwrap()),
            || {
                let (dir, removed) = clear_skillscout_cache();
                assert_eq!(dir, cache_root);
                assert!(!removed);
            },
        );
    }

    #[test]
    fn autoskills_alias_matches_skillscout() {
        with_env_var("SKILLSCOUT_CACHE_DIR", Some("/tmp/alias-test"), || {
            assert_eq!(get_autoskills_cache_dir(), get_skillscout_cache_dir());
        });
    }

    #[test]
    fn clear_autoskills_alias_works() {
        let tmp = tempdir().unwrap();
        let cache_root = tmp.path().join("alias-clear");
        fs::create_dir_all(&cache_root).unwrap();
        with_env_var(
            "SKILLSCOUT_CACHE_DIR",
            Some(cache_root.to_str().unwrap()),
            || {
                let (dir, removed) = clear_autoskills_cache();
                assert_eq!(dir, cache_root);
                assert!(removed);
            },
        );
    }

    #[test]
    fn get_cache_registry_dir_respects_env() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("cache-env-test");
        with_env_var("SKILLSCOUT_CACHE_DIR", Some(root.to_str().unwrap()), || {
            with_env_var("AUTOSKILLS_CACHE_DIR", None, || {
                let d = get_cache_registry_dir("hash123");
                assert_eq!(d, root.join("hash123"));
            });
        });
    }
}
