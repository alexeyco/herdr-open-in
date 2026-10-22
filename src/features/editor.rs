//! Runs a selected terminal editor inside its herdr tab.

use crate::adapters::editors::{self, DIR_PLACEHOLDER, Editor};
use anyhow::{Result, bail};

#[cfg_attr(test, mockall::automock)]
pub(crate) trait EditorRunner {
    fn run(&self, editor: &Editor, argv: &[String]) -> Result<()>;
}

pub(crate) fn run(runner: &impl EditorRunner, editor_id: &str, dir: &str) -> Result<()> {
    let editor = editor_for_id(editor_id)?;
    runner.run(editor, &editor_argv(editor, dir))
}

fn editor_for_id(id: &str) -> Result<&'static Editor> {
    if let Some(editor) = editors::CATALOG.iter().find(|editor| editor.id == id) {
        return Ok(editor);
    }
    let known_ids = editors::CATALOG
        .iter()
        .map(|editor| editor.id)
        .collect::<Vec<_>>()
        .join(", ");
    bail!("unknown editor `{id}`; known editors: {known_ids}")
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_lookup_covers_the_catalog() {
        for editor in editors::CATALOG {
            assert_eq!(editor_for_id(editor.id).unwrap().id, editor.id);
        }
    }

    #[test]
    fn unknown_editor_error_lists_known_ids() {
        let error = editor_for_id("not-an-editor").unwrap_err();
        for editor in editors::CATALOG {
            assert!(error.to_string().contains(editor.id));
        }
    }

    #[test]
    fn editor_arguments_substitute_the_directory() {
        let editor = editor_for_id("nvim").unwrap();
        assert_eq!(editor_argv(editor, "/work/demo"), ["/work/demo"]);
    }

    #[test]
    fn run_resolves_the_editor_and_passes_arguments_to_the_host() {
        let mut runner = MockEditorRunner::new();
        runner
            .expect_run()
            .withf(|editor, argv| editor.id == "nvim" && argv == ["/work/demo"])
            .returning(|_, _| Ok(()));
        run(&runner, "nvim", "/work/demo").unwrap();
    }

    #[test]
    fn unknown_editor_does_not_call_the_host() {
        let mut runner = MockEditorRunner::new();
        runner.expect_run().never();
        assert!(run(&runner, "unknown", "/work/demo").is_err());
    }
}
