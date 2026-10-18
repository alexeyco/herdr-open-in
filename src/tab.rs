//! Opening a terminal editor in a new herdr tab.
//!
//! Terminal editors (nvim, vim and helix) run inside herdr, so the picker asks herdr to
//! open a `tab`-placement pane running this plugin's `editor` entrypoint. The
//! chosen editor id and the resolved directory travel as `OPEN_IN_EDITOR` and
//! `OPEN_IN_DIR`, custom names because herdr does not let `--env` override its
//! managed `HERDR_*` variables. Plugin pane commands are argv-exec with no
//! shell, and pane mode must dispatch on `HERDR_PLUGIN_ENTRYPOINT_ID`, which
//! `--entrypoint editor` sets.

use std::process::Command;

use serde_json::Value;

use crate::editors::Editor;

/// Absolute path of the herdr binary that spawned this popup. A bare `herdr` is
/// never assumed: inside a popup the process environment carries this variable.
const HERDR_BIN_ENV: &str = "HERDR_BIN_PATH";

const PLUGIN_ID: &str = "alexeyco.open-in";
const ENTRYPOINT_ID: &str = "editor";
const PICKER_ENTRYPOINT_ID: &str = "picker";
const EDITOR_ENV: &str = "OPEN_IN_EDITOR";
const DIR_ENV: &str = "OPEN_IN_DIR";

/// Opens a new terminal tab in `workspace_id` running the plugin `editor`
/// entrypoint, with `editor.id` and `dir` passed through as environment.
///
/// The new tab is fully owned by herdr: this function only asks for it and
/// checks that herdr acknowledged the request with a pane id, then returns.
pub(crate) fn open_terminal_tab(
    editor: &Editor,
    workspace_id: &str,
    dir: &str,
) -> Result<(), String> {
    let argv = pane_open_argv(workspace_id, editor.id, dir);
    let stdout = run_pane_open(&argv)?;
    parse_pane_open_response(&stdout).map(|_pane_id| ())
}

/// Opens the picker as a focused `popup` pane. Returns once herdr has accepted
/// the request; a popup response carries no pane id, so only the exit status is
/// checked. A `ui_busy` rejection (another popup is already open) surfaces as
/// herdr's own error text.
pub(crate) fn open_picker_popup() -> Result<(), String> {
    run_pane_open(&picker_popup_argv()).map(|_stdout| ())
}

/// Runs `plugin pane open` with `argv` and returns its stdout on success. A
/// failed exit surfaces herdr's own `error` text when the response carries it.
fn run_pane_open(argv: &[String]) -> Result<String, String> {
    let bin = herdr_bin_path()?;
    let output = Command::new(&bin)
        .args(argv)
        .output()
        .map_err(|e| format!("failed to run {bin}: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        if let Some(message) = pane_open_error(&stdout) {
            return Err(message);
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "{bin} plugin pane open exited with {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(stdout.into_owned())
}

/// The exact argv handed to `$HERDR_BIN_PATH`, kept pure so it can be tested
/// without herdr. Plugin pane commands are argv-exec, so every value travels as
/// its own element and no shell quoting is involved.
fn pane_open_argv(workspace_id: &str, editor_id: &str, dir: &str) -> Vec<String> {
    vec![
        "plugin".to_string(),
        "pane".to_string(),
        "open".to_string(),
        "--plugin".to_string(),
        PLUGIN_ID.to_string(),
        "--entrypoint".to_string(),
        ENTRYPOINT_ID.to_string(),
        "--placement".to_string(),
        "tab".to_string(),
        "--workspace".to_string(),
        workspace_id.to_string(),
        "--cwd".to_string(),
        dir.to_string(),
        "--env".to_string(),
        format!("{EDITOR_ENV}={editor_id}"),
        "--env".to_string(),
        format!("{DIR_ENV}={dir}"),
    ]
}

/// The exact argv for the picker popup, kept pure so it can be tested without
/// herdr. Plugin pane commands are argv-exec, so every value travels as its
/// own element and no shell quoting is involved.
fn picker_popup_argv() -> Vec<String> {
    [
        "plugin",
        "pane",
        "open",
        "--plugin",
        PLUGIN_ID,
        "--entrypoint",
        PICKER_ENTRYPOINT_ID,
        "--placement",
        "popup",
        "--focus",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// Resolves `$HERDR_BIN_PATH`; unset, empty or whitespace-only is an error
/// naming the variable.
fn herdr_bin_path() -> Result<String, String> {
    herdr_bin_path_from(std::env::var(HERDR_BIN_ENV).ok().as_deref())
}

/// [`herdr_bin_path`] against an already-read env value, so tests can exercise
/// the error path without touching the process environment.
fn herdr_bin_path_from(raw: Option<&str>) -> Result<String, String> {
    match raw.map(str::trim) {
        Some(path) if !path.is_empty() => Ok(path.to_string()),
        _ => Err(format!(
            "{HERDR_BIN_ENV} is not set or blank; invoke this action from herdr"
        )),
    }
}

/// Parses the `plugin pane open` response for the success path, returning the
/// new pane id read from `result.plugin_pane.pane.pane_id`. An `error` object, a
/// blank pane id and malformed JSON are all errors carrying herdr's wording.
fn parse_pane_open_response(stdout: &str) -> Result<String, String> {
    let v: Value =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("invalid herdr response: {e}"))?;
    if let Some(message) = error_message(&v) {
        return Err(message);
    }
    v.pointer("/result/plugin_pane/pane/pane_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "herdr response has no result.plugin_pane.pane.pane_id: {}",
                stdout.trim()
            )
        })
}

/// The `error` object of a response, if any, as an actionable message.
fn pane_open_error(stdout: &str) -> Option<String> {
    let v: Value = serde_json::from_str(stdout.trim()).ok()?;
    error_message(&v)
}

/// Formats herdr's `error` value, whether it is an object with `code`/`message`
/// or a bare string. `None` when the response has no `error` key.
fn error_message(v: &Value) -> Option<String> {
    let error = v.get("error")?;
    if let Some(text) = error.as_str() {
        return Some(format!("herdr reported error: {text}"));
    }
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let message = error.get("message").and_then(Value::as_str).unwrap_or("");
    if message.is_empty() {
        Some(format!("herdr reported error {code}"))
    } else {
        Some(format!("herdr reported error {code}: {message}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_matches_the_documented_command_in_order() {
        let argv = pane_open_argv("ws-1", "nvim", "/home/u/demo");
        assert_eq!(
            argv,
            [
                "plugin",
                "pane",
                "open",
                "--plugin",
                "alexeyco.open-in",
                "--entrypoint",
                "editor",
                "--placement",
                "tab",
                "--workspace",
                "ws-1",
                "--cwd",
                "/home/u/demo",
                "--env",
                "OPEN_IN_EDITOR=nvim",
                "--env",
                "OPEN_IN_DIR=/home/u/demo",
            ]
        );
    }

    #[test]
    fn argv_env_flags_carry_the_editor_id_and_dir() {
        let argv = pane_open_argv("ws-7", "helix", "/tmp/x y");
        let env_values: Vec<&str> = argv
            .iter()
            .enumerate()
            .filter(|(_, arg)| arg.as_str() == "--env")
            .filter_map(|(i, _)| argv.get(i + 1).map(String::as_str))
            .collect();
        assert_eq!(env_values, ["OPEN_IN_EDITOR=helix", "OPEN_IN_DIR=/tmp/x y"]);
    }

    #[test]
    fn argv_workspace_and_cwd_immediately_follow_their_flags() {
        let argv = pane_open_argv("ws-42", "nvim", "/srv/repo");
        let workspace = argv
            .iter()
            .position(|arg| arg == "--workspace")
            .and_then(|i| argv.get(i + 1));
        let cwd = argv
            .iter()
            .position(|arg| arg == "--cwd")
            .and_then(|i| argv.get(i + 1));
        assert_eq!(workspace.map(String::as_str), Some("ws-42"));
        assert_eq!(cwd.map(String::as_str), Some("/srv/repo"));
    }

    #[test]
    fn picker_popup_argv_matches_the_documented_command_in_order() {
        assert_eq!(
            picker_popup_argv(),
            [
                "plugin",
                "pane",
                "open",
                "--plugin",
                "alexeyco.open-in",
                "--entrypoint",
                "picker",
                "--placement",
                "popup",
                "--focus",
            ]
        );
    }

    #[test]
    fn missing_herdr_bin_path_is_an_error_naming_the_variable() {
        let err = match herdr_bin_path_from(None) {
            Ok(path) => panic!("expected an error, got {path}"),
            Err(e) => e,
        };
        assert!(err.contains(HERDR_BIN_ENV), "unhelpful error: {err}");
    }

    #[test]
    fn blank_herdr_bin_path_is_an_error() {
        for raw in ["", "   ", "\t\n"] {
            assert!(
                herdr_bin_path_from(Some(raw)).is_err(),
                "expected an error for {raw:?}"
            );
        }
    }

    #[test]
    fn herdr_bin_path_is_trimmed() {
        assert_eq!(
            herdr_bin_path_from(Some("  /usr/local/bin/herdr  ")).as_deref(),
            Ok("/usr/local/bin/herdr")
        );
    }

    #[test]
    fn success_response_yields_the_new_pane_id() {
        let stdout =
            r#"{"result":{"plugin_pane":{"pane":{"pane_id":"pane-42"},"tab_id":"tab-1"}}}"#;
        assert_eq!(parse_pane_open_response(stdout).as_deref(), Ok("pane-42"));
    }

    #[test]
    fn error_response_reports_the_code_and_message() {
        let stdout = r#"{"error":{"code":"plugin_not_found","message":"no such plugin"}}"#;
        let err = match parse_pane_open_response(stdout) {
            Ok(pane) => panic!("expected an error, got {pane}"),
            Err(e) => e,
        };
        assert!(err.contains("plugin_not_found"), "unhelpful error: {err}");
        assert!(err.contains("no such plugin"), "unhelpful error: {err}");
        assert_eq!(pane_open_error(stdout).as_deref(), Some(err.as_str()));
    }

    #[test]
    fn success_response_without_a_pane_id_is_an_error() {
        let stdout = r#"{"result":{"type":"ok"}}"#;
        let err = match parse_pane_open_response(stdout) {
            Ok(pane) => panic!("expected an error, got {pane}"),
            Err(e) => e,
        };
        assert!(err.contains("pane_id"), "unhelpful error: {err}");
        assert_eq!(pane_open_error(stdout), None);
    }

    #[test]
    fn blank_pane_id_is_an_error() {
        let stdout = r#"{"result":{"plugin_pane":{"pane":{"pane_id":"  "}}}}"#;
        assert!(parse_pane_open_response(stdout).is_err());
    }

    #[test]
    fn malformed_response_is_an_error() {
        assert!(
            parse_pane_open_response("{not json")
                .unwrap_err()
                .contains("invalid")
        );
        assert!(parse_pane_open_response("").is_err());
        assert_eq!(pane_open_error("{not json"), None);
    }

    #[test]
    fn string_error_response_is_reported() {
        let stdout = r#"{"error":"ui_busy"}"#;
        let err = match parse_pane_open_response(stdout) {
            Ok(pane) => panic!("expected an error, got {pane}"),
            Err(e) => e,
        };
        assert!(err.contains("ui_busy"), "unhelpful error: {err}");
    }
}
