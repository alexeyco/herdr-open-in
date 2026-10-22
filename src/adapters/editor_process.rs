//! Process adapter for the terminal-editor entrypoint.

use std::process::Command;

use crate::adapters::editors::Editor;
use crate::features::editor::{self, EditorRunner};
use anyhow::{Context, Result, bail};

pub(crate) fn run(editor_id: &str, dir: &str) -> Result<()> {
    editor::run(&Process, editor_id, dir)
}

struct Process;

impl EditorRunner for Process {
    fn run(&self, editor: &Editor, argv: &[String]) -> Result<()> {
        let status = Command::new(editor.cmd)
            .args(argv)
            .status()
            .with_context(|| format!("failed to launch {} (`{}`)", editor.name, editor.cmd))?;
        if status.success() {
            Ok(())
        } else {
            bail!("{} exited with {status}", editor.name)
        }
    }
}
