//! herdr plugin that opens the current workspace in the editor you pick.
//!
//! The `open-in.open` action opens the picker as a focused `popup` pane running
//! the `picker` entrypoint; that pane lists every catalog editor detected on
//! this machine and, on Enter, launches the choice: a GUI editor is started
//! detached on the resolved directory, while a terminal editor (nvim, vim and helix)
//! opens a new herdr `tab` running this same binary's `editor` entrypoint
//! inside the current workspace.
//!
//! Dispatch is driven by the entrypoint environment variable, which wins over
//! the action id: `picker` shows the dialog, `editor` execs the chosen terminal
//! editor in the foreground, and when only the action id is set the binary
//! opens the picker popup itself.
//!
//! Plugin docs: https://herdr.dev/docs/plugins/

mod context;
mod editors;
mod launch;
mod picker;
mod state;
mod tab;
mod theme;

use std::process::Command;

use editors::{DIR_PLACEHOLDER, Editor, EditorKind};

/// Entrypoint environment variable, set by herdr when it runs a plugin pane.
const ENTRYPOINT_ENV: &str = "HERDR_PLUGIN_ENTRYPOINT_ID";

/// Action environment variable, set by herdr when it runs a plugin action.
const ACTION_ENV: &str = "HERDR_PLUGIN_ACTION_ID";

/// Environment variable carrying the plugin context JSON.
const CONTEXT_ENV: &str = "HERDR_PLUGIN_CONTEXT_JSON";

/// Entrypoint id of the picker popup.
const PICKER_ENTRYPOINT: &str = "picker";

/// Entrypoint id of the terminal-editor tab pane.
const EDITOR_ENTRYPOINT: &str = "editor";

/// Environment variable carrying the chosen editor id into the tab pane.
const EDITOR_ENV: &str = "OPEN_IN_EDITOR";

/// Environment variable carrying the directory into the tab pane.
const DIR_ENV: &str = "OPEN_IN_DIR";

fn main() {
    if let Err(e) = run() {
        eprintln!("open-in: {e}");
        std::process::exit(1);
    }
}

/// How this process was invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// `HERDR_PLUGIN_ENTRYPOINT_ID=picker`: show the popup dialog.
    Picker,
    /// `HERDR_PLUGIN_ENTRYPOINT_ID=editor`: exec the terminal editor in a tab.
    Editor,
    /// Only `HERDR_PLUGIN_ACTION_ID` is set: open the picker popup.
    Action,
    /// Neither variable came from herdr.
    Unbound,
    /// An entrypoint other than `picker` or `editor`.
    Unknown,
}

fn run() -> Result<(), String> {
    let entrypoint = env_var(ENTRYPOINT_ENV);
    let action = env_var(ACTION_ENV);
    match mode_from(entrypoint.as_deref(), action.as_deref()) {
        Mode::Picker => run_picker(),
        Mode::Editor => run_editor(),
        Mode::Action => tab::open_picker_popup(),
        Mode::Unknown => Err(format!(
            "unknown plugin entrypoint `{}`; expected `{PICKER_ENTRYPOINT}` or `{EDITOR_ENTRYPOINT}`",
            entrypoint.as_deref().unwrap_or_default()
        )),
        Mode::Unbound => Err(format!(
            "neither {ENTRYPOINT_ENV} nor {ACTION_ENV} is set; \
             bind `open-in.open` to a key in herdr or invoke it from herdr"
        )),
    }
}

/// Maps the entrypoint and action environment to the mode to run. A set
/// entrypoint wins over the action id.
fn mode_from(entrypoint: Option<&str>, action: Option<&str>) -> Mode {
    match entrypoint {
        Some(id) if id == PICKER_ENTRYPOINT => Mode::Picker,
        Some(id) if id == EDITOR_ENTRYPOINT => Mode::Editor,
        Some(_) => Mode::Unknown,
        None => match action {
            Some(_) => Mode::Action,
            None => Mode::Unbound,
        },
    }
}

/// The picker flow: resolve the context, detect editors, show the dialog, and
/// launch the chosen editor.
fn run_picker() -> Result<(), String> {
    let context = required_env(CONTEXT_ENV)?;
    let dir = context::dir_from_context(&context)?;
    let workspace_id = context::workspace_id_from_context(&context)?;

    let items = editors::detect();
    if items.is_empty() {
        return Err(no_editors_message());
    }

    // Always show the dialog, even for a single editor (AC6). `None` means the
    // user cancelled with Escape or `q`, which is a clean exit.
    let preselect = state::last_choice();
    let editor = match picker::run(&items, preselect.as_deref())? {
        Some(editor) => editor,
        None => return Ok(()),
    };

    // Persisting the choice is best-effort: a failure is a warning, never a
    // reason to skip the launch the user just asked for.
    if let Err(e) = state::save_choice(editor.id) {
        eprintln!("open-in: warning: could not save the last editor: {e}");
    }

    launch(editor, &dir, &workspace_id)
}

/// Launches the chosen editor: GUI detached, terminal in a new herdr tab.
fn launch(editor: &Editor, dir: &str, workspace_id: &str) -> Result<(), String> {
    match editor.kind {
        EditorKind::Gui => launch::launch_gui(editor, dir),
        EditorKind::Terminal => tab::open_terminal_tab(editor, workspace_id, dir),
    }
}

/// The tab-pane flow: exec the chosen editor in the foreground so it owns the
/// tty until it exits.
fn run_editor() -> Result<(), String> {
    let editor_id = required_env(EDITOR_ENV)?;
    let dir = required_env(DIR_ENV)?;
    let editor = editor_for_id(&editor_id)?;

    let status = Command::new(editor.cmd)
        .args(editor_argv(editor, &dir))
        .status()
        .map_err(|e| format!("failed to launch {} (`{}`): {e}", editor.name, editor.cmd))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {status}", editor.name))
    }
}

/// Looks up a catalog row by its stable id, or an error listing the known ids.
fn editor_for_id(id: &str) -> Result<&'static Editor, String> {
    editor_by_id(id).ok_or_else(|| {
        format!(
            "unknown editor `{id}`; known editors: {}",
            known_editor_ids()
        )
    })
}

/// Finds a catalog row by its stable id.
fn editor_by_id(id: &str) -> Option<&'static Editor> {
    editors::CATALOG.iter().find(|editor| editor.id == id)
}

/// Comma-separated catalog ids, for error messages.
fn known_editor_ids() -> String {
    editors::CATALOG
        .iter()
        .map(|editor| editor.id)
        .collect::<Vec<_>>()
        .join(", ")
}

/// argv for a terminal editor, substituting the directory placeholder.
fn editor_argv(editor: &Editor, dir: &str) -> Vec<String> {
    editor
        .args
        .iter()
        .map(|arg| {
            if *arg == DIR_PLACEHOLDER {
                dir.to_string()
            } else {
                (*arg).to_string()
            }
        })
        .collect()
}

/// The single error line for a machine with no catalog editor installed.
fn no_editors_message() -> String {
    let names: Vec<&str> = editors::CATALOG.iter().map(|editor| editor.name).collect();
    format!(
        "no supported editor was found; install one of: {} \
         (its CLI on PATH or, on macOS, its app bundle in /Applications or ~/Applications)",
        names.join(", ")
    )
}

/// Reads an environment variable, treating unset, non-UTF-8 and blank values
/// as absent.
fn env_var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Reads a required environment variable; unset or blank names the variable.
fn required_env(name: &str) -> Result<String, String> {
    env_var(name).ok_or_else(|| format!("{name} is not set or blank"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_from_prefers_the_entrypoint_over_the_action() {
        assert_eq!(mode_from(Some("picker"), Some("open")), Mode::Picker);
        assert_eq!(mode_from(Some("editor"), Some("open")), Mode::Editor);
    }

    #[test]
    fn mode_from_falls_back_to_the_action_id() {
        assert_eq!(mode_from(None, Some("open")), Mode::Action);
    }

    #[test]
    fn mode_from_is_unbound_without_entrypoint_or_action() {
        assert_eq!(mode_from(None, None), Mode::Unbound);
    }

    #[test]
    fn mode_from_rejects_an_unknown_entrypoint_even_with_an_action() {
        assert_eq!(mode_from(Some("bogus"), Some("open")), Mode::Unknown);
        assert_eq!(mode_from(Some("bogus"), None), Mode::Unknown);
    }

    #[test]
    fn editor_by_id_finds_every_catalog_row() {
        for editor in editors::CATALOG {
            assert_eq!(
                editor_by_id(editor.id).map(|found| found.id),
                Some(editor.id)
            );
        }
    }

    #[test]
    fn editor_by_id_rejects_an_unknown_id() {
        assert!(editor_by_id("not-an-editor").is_none());
    }

    #[test]
    fn editor_for_id_error_lists_the_known_ids() {
        let err = match editor_for_id("not-an-editor") {
            Ok(editor) => panic!("expected an error, got {}", editor.id),
            Err(e) => e,
        };
        for editor in editors::CATALOG {
            assert!(
                err.contains(editor.id),
                "error is missing {}: {err}",
                editor.id
            );
        }
    }

    #[test]
    fn editor_argv_substitutes_the_directory() {
        let editor = match editor_by_id("nvim") {
            Some(editor) => editor,
            None => panic!("nvim catalog row is missing"),
        };
        assert_eq!(editor_argv(editor, "/home/u/demo"), ["/home/u/demo"]);
    }

    #[test]
    fn no_editors_message_names_every_catalog_editor() {
        let message = no_editors_message();
        for editor in editors::CATALOG {
            assert!(
                message.contains(editor.name),
                "message is missing {}: {message}",
                editor.name
            );
        }
    }
}
