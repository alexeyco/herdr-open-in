//! Runtime adapters for opening a workspace in an editor.

use super::{context, editors, environment, launch, picker, state, tab};
use crate::adapters::editors::Editor;
use crate::features::open_in::{OpenInHost, Workspace};
use anyhow::{Context, Result};

const CONTEXT_ENV: &str = "HERDR_PLUGIN_CONTEXT_JSON";

pub(crate) fn run() -> Result<()> {
    crate::features::open_in::run(&Host)
}

struct Host;

impl OpenInHost for Host {
    fn workspace(&self) -> Result<Workspace> {
        let context = environment::required(CONTEXT_ENV)?;
        Ok(Workspace {
            dir: context::dir_from_context(&context)
                .map_err(anyhow::Error::msg)
                .context("cannot resolve workspace directory")?,
            id: context::workspace_id_from_context(&context)
                .map_err(anyhow::Error::msg)
                .context("cannot resolve workspace id")?,
        })
    }

    fn editors(&self) -> Vec<&'static Editor> {
        editors::detect()
    }

    fn last_choice(&self) -> Option<String> {
        state::last_choice()
    }

    fn pick(
        &self,
        editors: &[&'static Editor],
        preselect: Option<String>,
    ) -> Result<Option<&'static Editor>> {
        picker::run(editors, preselect.as_deref()).map_err(anyhow::Error::msg)
    }

    fn save_choice(&self, editor_id: &str) -> Result<()> {
        state::save_choice(editor_id).map_err(anyhow::Error::msg)
    }

    fn launch(&self, editor: &'static Editor, workspace: &Workspace) -> Result<()> {
        let result = match editor.kind {
            editors::EditorKind::Gui => launch::launch_gui(editor, &workspace.dir),
            editors::EditorKind::Terminal => {
                tab::open_terminal_tab(editor, &workspace.id, &workspace.dir)
            }
        };
        result.map_err(anyhow::Error::msg)
    }

    fn warn(&self, message: &str) {
        eprintln!("open-in: warning: {message}");
    }
}
