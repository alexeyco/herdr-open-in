//! herdr plugin that opens the current workspace in the editor you pick.
//!
//! The `alexeyco.open-in.open` action opens the picker as a focused popup.
//! One detected editor launches directly; several editors open the picker.
//! GUI editors start detached, while terminal editors open in a herdr tab.

use anyhow::{Result, anyhow, bail};

mod adapters;
mod features;

const ENTRYPOINT_ENV: &str = "HERDR_PLUGIN_ENTRYPOINT_ID";
const ACTION_ENV: &str = "HERDR_PLUGIN_ACTION_ID";
const EDITOR_ENV: &str = "OPEN_IN_EDITOR";
const DIR_ENV: &str = "OPEN_IN_DIR";
const PICKER_ENTRYPOINT: &str = "picker";
const EDITOR_ENTRYPOINT: &str = "editor";

fn main() {
    if let Err(error) = run() {
        eprintln!("open-in: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let entrypoint = adapters::environment::value(ENTRYPOINT_ENV);
    let action = adapters::environment::value(ACTION_ENV);
    match mode_from(entrypoint.as_deref(), action.as_deref()) {
        Mode::Picker => adapters::open_in::run(),
        Mode::Editor => adapters::editor_process::run(
            &adapters::environment::required(EDITOR_ENV)?,
            &adapters::environment::required(DIR_ENV)?,
        ),
        Mode::Action => adapters::tab::open_picker_popup().map_err(anyhow::Error::msg),
        Mode::Unknown => Err(anyhow!(
            "unknown plugin entrypoint `{}`; expected `{PICKER_ENTRYPOINT}` or `{EDITOR_ENTRYPOINT}`",
            entrypoint.as_deref().unwrap_or_default()
        )),
        Mode::Unbound => bail!(
            "neither {ENTRYPOINT_ENV} nor {ACTION_ENV} is set; \
             bind `alexeyco.open-in.open` to a key in herdr or invoke it from herdr"
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Picker,
    Editor,
    Action,
    Unbound,
    Unknown,
}

fn mode_from(entrypoint: Option<&str>, action: Option<&str>) -> Mode {
    match entrypoint {
        Some(PICKER_ENTRYPOINT) => Mode::Picker,
        Some(EDITOR_ENTRYPOINT) => Mode::Editor,
        Some(_) => Mode::Unknown,
        None if action.is_some() => Mode::Action,
        None => Mode::Unbound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrypoint_takes_precedence_over_action() {
        assert_eq!(mode_from(Some("picker"), Some("open")), Mode::Picker);
        assert_eq!(mode_from(Some("editor"), Some("open")), Mode::Editor);
        assert_eq!(mode_from(Some("bogus"), Some("open")), Mode::Unknown);
    }

    #[test]
    fn action_selects_action_mode_only_without_entrypoint() {
        assert_eq!(mode_from(None, Some("open")), Mode::Action);
        assert_eq!(mode_from(None, None), Mode::Unbound);
    }
}
