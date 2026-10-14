//! Persisting the last picked editor id under `HERDR_PLUGIN_STATE_DIR`.
//!
//! herdr creates the per-plugin state directory (`~/.local/state/herdr/plugins/<id>`)
//! but owns no file format inside it, so the choice lives in the small file
//! `<state dir>/last-editor` holding the bare editor id and a trailing newline.

use std::path::{Path, PathBuf};

/// Environment variable holding the per-plugin state directory created by herdr.
const STATE_DIR_ENV: &str = "HERDR_PLUGIN_STATE_DIR";

/// Name of the file holding the last chosen editor id.
const LAST_EDITOR_FILE: &str = "last-editor";

/// Reads the last picked editor id from `$HERDR_PLUGIN_STATE_DIR/last-editor`.
/// Returns `None` when the directory env var is unset or blank, the file is
/// missing, or it contains only whitespace.
pub(crate) fn last_choice() -> Option<String> {
    let raw = std::env::var(STATE_DIR_ENV).ok();
    last_choice_from(raw.as_deref())
}

/// Writes `editor_id` to `$HERDR_PLUGIN_STATE_DIR/last-editor`.
/// Errors when the directory env var is unset or blank, or the write fails;
/// the caller treats a failure as a warning, never a panic.
pub(crate) fn save_choice(editor_id: &str) -> Result<(), String> {
    let raw = std::env::var(STATE_DIR_ENV).ok();
    save_choice_to(raw.as_deref(), editor_id)
}

/// Resolves the state directory from a raw env value; unset or blank yields `None`.
fn dir_from(raw: Option<&str>) -> Option<PathBuf> {
    let raw = raw?.trim();
    if raw.is_empty() {
        None
    } else {
        Some(PathBuf::from(raw))
    }
}

/// `last_choice` against an already-resolved raw env value, without touching the process env.
fn last_choice_from(raw: Option<&str>) -> Option<String> {
    let dir = dir_from(raw)?;
    read_from(&dir)
}

/// `save_choice` against an already-resolved raw env value, without touching the process env.
fn save_choice_to(raw: Option<&str>, editor_id: &str) -> Result<(), String> {
    let dir = dir_from(raw).ok_or_else(|| {
        format!(
            "{STATE_DIR_ENV} is not set or blank; \
             invoke this action from herdr so the last editor choice can be saved"
        )
    })?;
    write_to(&dir, editor_id)
}

/// Reads the bare editor id from `dir/last-editor`, trimming surrounding
/// whitespace. A missing, unreadable or blank file yields `None`.
fn read_from(dir: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(dir.join(LAST_EDITOR_FILE)).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Writes `id` to `dir/last-editor` via a sibling temp file and a rename, so a
/// reader never observes a half-written file. Creates `dir` when missing.
fn write_to(dir: &Path, id: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("cannot create state dir {}: {e}", dir.display()))?;
    let tmp = dir.join(format!("{LAST_EDITOR_FILE}.{}.tmp", std::process::id()));
    std::fs::write(&tmp, format!("{id}\n"))
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, dir.join(LAST_EDITOR_FILE)).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("cannot save the last editor in {}: {e}", dir.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    /// A uniquely named directory under the system temp dir, removed on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> Self {
            let unique = format!(
                "herdr-open-in-state-{}-{}-{}",
                std::process::id(),
                tag,
                COUNTER.fetch_add(1, Ordering::Relaxed)
            );
            let path = std::env::temp_dir().join(unique);
            let _ = std::fs::remove_dir_all(&path);
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn raw(&self) -> Option<&str> {
            self.0.to_str()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_file_is_none() {
        let dir = TempDir::new("missing");
        assert_eq!(read_from(dir.path()), None);
    }

    #[test]
    fn round_trip_write_then_read() {
        let dir = TempDir::new("roundtrip");
        assert!(write_to(dir.path(), "zed").is_ok());
        assert_eq!(read_from(dir.path()).as_deref(), Some("zed"));
    }

    #[test]
    fn round_trip_through_env_helpers() {
        let dir = TempDir::new("helpers");
        assert!(save_choice_to(dir.raw(), "android-studio").is_ok());
        assert_eq!(
            last_choice_from(dir.raw()).as_deref(),
            Some("android-studio")
        );
    }

    #[test]
    fn blank_file_is_none() {
        let dir = TempDir::new("blank-file");
        assert!(write_to(dir.path(), "   ").is_ok());
        assert_eq!(read_from(dir.path()), None);
    }

    #[test]
    fn whitespace_is_trimmed_on_read() {
        let dir = TempDir::new("trim");
        assert!(write_to(dir.path(), "  zed  ").is_ok());
        assert_eq!(read_from(dir.path()).as_deref(), Some("zed"));
    }

    #[test]
    fn blank_or_unset_state_dir_yields_none_or_error() {
        assert_eq!(last_choice_from(None), None);
        assert_eq!(last_choice_from(Some("  ")), None);
        assert!(save_choice_to(None, "zed").is_err());
        assert!(save_choice_to(Some("\t"), "zed").is_err());
    }

    #[test]
    fn unset_state_dir_error_names_the_variable() {
        let err = match save_choice_to(None, "zed") {
            Ok(()) => panic!("expected an error for an unset state dir"),
            Err(e) => e,
        };
        assert!(err.contains(STATE_DIR_ENV), "unhelpful error: {err}");
    }
}
