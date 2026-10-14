//! Resolving the open directory from the herdr plugin context JSON.

use serde_json::Value;

/// Picks the first present, non-blank string along the resolution chain:
/// `worktree.checkout_path` → `workspace_cwd` → `focused_pane_cwd`.
/// A non-string or whitespace-only value counts as absent.
pub(crate) fn dir_from_context(json: &str) -> Result<String, String> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| format!("invalid plugin context JSON: {e}"))?;
    fn non_blank(s: &str) -> Option<&str> {
        let s = s.trim();
        if s.is_empty() { None } else { Some(s) }
    }
    let dir = v
        .get("worktree")
        .and_then(|w| w.get("checkout_path"))
        .and_then(Value::as_str)
        .and_then(non_blank)
        .or_else(|| {
            v.get("workspace_cwd")
                .and_then(Value::as_str)
                .and_then(non_blank)
        })
        .or_else(|| {
            v.get("focused_pane_cwd")
                .and_then(Value::as_str)
                .and_then(non_blank)
        })
        .ok_or_else(|| {
            "no directory in plugin context; invoke this action inside a workspace".to_string()
        })?;
    Ok(dir.to_string())
}

/// Reads the `workspace_id` from the plugin context JSON.
/// A missing, non-string or whitespace-only value is an error.
pub(crate) fn workspace_id_from_context(json: &str) -> Result<String, String> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| format!("invalid plugin context JSON: {e}"))?;
    let id = v
        .get("workspace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "no workspace_id in plugin context; invoke this action inside a workspace".to_string()
        })?;
    Ok(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTX: &str = concat!(
        r#"{"workspace_id":"ws-1","workspace_label":"demo","tab_id":"t1","#,
        r#""worktree":{"repo_key":"github.com/u/demo","repo_name":"demo","#,
        r#""repo_root":"/home/u/demo","checkout_path":"/home/u/demo","is_linked_worktree":false},"#,
        r#""workspace_cwd":"/home/u/demo/src","focused_pane_id":"p1","#,
        r#""focused_pane_cwd":"/home/u/demo/src/tests"}"#
    );

    #[test]
    fn full_context_prefers_checkout_path() {
        assert_eq!(dir_from_context(CTX).as_deref(), Ok("/home/u/demo"));
    }

    #[test]
    fn falls_back_to_workspace_cwd() {
        let json = r#"{"worktree":{},"workspace_cwd":"/home/u/demo/src"}"#;
        assert_eq!(dir_from_context(json).as_deref(), Ok("/home/u/demo/src"));
    }

    #[test]
    fn non_string_value_is_absent() {
        for json in [r#"{"workspace_cwd":42}"#, r#"{"workspace_cwd":null}"#] {
            assert!(dir_from_context(json).unwrap_err().contains("no directory"));
        }
    }

    #[test]
    fn empty_string_value_falls_through_to_next_candidate() {
        let json = r#"{"worktree":{"checkout_path":"  "},"workspace_cwd":"/home/u/demo"}"#;
        assert_eq!(dir_from_context(json).as_deref(), Ok("/home/u/demo"));
    }

    #[test]
    fn all_blank_values_are_an_error() {
        for json in [
            r#"{"workspace_cwd":""}"#,
            r#"{"worktree":{"checkout_path":" "},"workspace_cwd":"","focused_pane_cwd":"\t"}"#,
        ] {
            assert!(dir_from_context(json).unwrap_err().contains("no directory"));
        }
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(
            dir_from_context("{not json")
                .unwrap_err()
                .contains("invalid")
        );
    }

    #[test]
    fn unescapes_windows_and_unicode_paths() {
        let json = r#"{"worktree":{"checkout_path":"C:\\Users\\caf\u00e9"}}"#;
        assert_eq!(dir_from_context(json).as_deref(), Ok(r"C:\Users\café"));
    }

    #[test]
    fn workspace_id_from_full_context() {
        assert_eq!(workspace_id_from_context(CTX).as_deref(), Ok("ws-1"));
    }

    #[test]
    fn blank_workspace_id_is_an_error() {
        assert!(
            workspace_id_from_context(r#"{"workspace_id":"  "}"#)
                .unwrap_err()
                .contains("no workspace_id")
        );
    }

    #[test]
    fn missing_workspace_id_is_an_error() {
        assert!(
            workspace_id_from_context(r#"{"tab_id":"t1"}"#)
                .unwrap_err()
                .contains("no workspace_id")
        );
    }

    #[test]
    fn non_string_workspace_id_is_an_error() {
        assert!(
            workspace_id_from_context(r#"{"workspace_id":42}"#)
                .unwrap_err()
                .contains("no workspace_id")
        );
    }

    #[test]
    fn invalid_json_workspace_id_is_an_error() {
        assert!(
            workspace_id_from_context("{not json")
                .unwrap_err()
                .contains("invalid")
        );
    }
}
