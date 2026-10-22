//! Application flow for choosing and opening an editor.

use crate::adapters::editors::{self, Editor};
use anyhow::{Result, bail};

pub(crate) struct Workspace {
    pub(crate) dir: String,
    pub(crate) id: String,
}

#[cfg_attr(test, mockall::automock)]
pub(crate) trait OpenInHost {
    fn workspace(&self) -> Result<Workspace>;
    fn editors(&self) -> Vec<&'static Editor>;
    fn last_choice(&self) -> Option<String>;
    fn pick(
        &self,
        editors: &[&'static Editor],
        preselect: Option<String>,
    ) -> Result<Option<&'static Editor>>;
    fn save_choice(&self, editor_id: &str) -> Result<()>;
    fn launch(&self, editor: &'static Editor, workspace: &Workspace) -> Result<()>;
    fn warn(&self, message: &str);
}

pub(crate) fn run(host: &impl OpenInHost) -> Result<()> {
    let workspace = host.workspace()?;
    let editors = host.editors();
    let Some(editor) = select_editor(host, &editors)? else {
        return Ok(());
    };

    if editors.len() > 1
        && let Err(error) = host.save_choice(editor.id)
    {
        host.warn(&format!("could not save the last editor: {error:#}"));
    }
    host.launch(editor, &workspace)
}

fn select_editor(
    host: &impl OpenInHost,
    editors: &[&'static Editor],
) -> Result<Option<&'static Editor>> {
    match editors {
        [] => bail!("{}", no_editors_message()),
        [editor] => Ok(Some(editor)),
        _ => {
            let last_choice = host.last_choice();
            host.pick(editors, last_choice)
        }
    }
}

fn no_editors_message() -> String {
    let names = editors::CATALOG
        .iter()
        .map(|editor| editor.name)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "no supported editor was found; install one of: {names} \
         (its CLI on PATH or, on macOS, its app bundle in /Applications or ~/Applications)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::Sequence;

    fn workspace() -> Workspace {
        Workspace {
            dir: "/work/demo".to_string(),
            id: "workspace-1".to_string(),
        }
    }

    #[test]
    fn one_editor_launches_without_picker_or_state_write() {
        let editor = &editors::CATALOG[0];
        let mut host = MockOpenInHost::new();
        let mut sequence = Sequence::new();
        host.expect_workspace()
            .in_sequence(&mut sequence)
            .returning(|| Ok(workspace()));
        host.expect_editors()
            .in_sequence(&mut sequence)
            .return_const(vec![editor]);
        host.expect_launch()
            .withf(move |selected, workspace| {
                selected.id == editor.id
                    && workspace.dir == "/work/demo"
                    && workspace.id == "workspace-1"
            })
            .in_sequence(&mut sequence)
            .returning(|_, _| Ok(()));
        run(&host).unwrap();
    }

    #[test]
    fn multiple_editors_preselect_save_best_effort_and_launch() {
        let items = &editors::CATALOG[..2];
        let selected = &items[1];
        let mut host = MockOpenInHost::new();
        let mut sequence = Sequence::new();
        host.expect_workspace()
            .in_sequence(&mut sequence)
            .returning(|| Ok(workspace()));
        host.expect_editors()
            .in_sequence(&mut sequence)
            .return_const(items.iter().collect::<Vec<_>>());
        host.expect_last_choice()
            .in_sequence(&mut sequence)
            .return_const(Some(selected.id.to_string()));
        host.expect_pick()
            .withf(move |available, preselect| {
                available.len() == 2 && preselect.as_deref() == Some(selected.id)
            })
            .in_sequence(&mut sequence)
            .returning(move |_, _| Ok(Some(selected)));
        host.expect_save_choice()
            .withf(move |id| id == selected.id)
            .in_sequence(&mut sequence)
            .returning(|_| Err(anyhow::anyhow!("state unavailable")));
        host.expect_warn()
            .withf(|message| message == "could not save the last editor: state unavailable")
            .in_sequence(&mut sequence)
            .return_const(());
        host.expect_launch()
            .withf(move |editor, _| editor.id == selected.id)
            .in_sequence(&mut sequence)
            .returning(|_, _| Ok(()));
        run(&host).unwrap();
    }

    #[test]
    fn cancellation_stops_before_save_and_launch() {
        let items = &editors::CATALOG[..2];
        let mut host = MockOpenInHost::new();
        let mut sequence = Sequence::new();
        host.expect_workspace()
            .in_sequence(&mut sequence)
            .returning(|| Ok(workspace()));
        host.expect_editors()
            .in_sequence(&mut sequence)
            .return_const(items.iter().collect::<Vec<_>>());
        host.expect_last_choice()
            .in_sequence(&mut sequence)
            .return_const(None);
        host.expect_pick()
            .in_sequence(&mut sequence)
            .returning(|_, _| Ok(None));
        run(&host).unwrap();
    }

    #[test]
    fn picker_error_propagates_without_state_write_or_launch() {
        let items = &editors::CATALOG[..2];
        let mut host = MockOpenInHost::new();
        let mut sequence = Sequence::new();
        host.expect_workspace()
            .in_sequence(&mut sequence)
            .returning(|| Ok(workspace()));
        host.expect_editors()
            .in_sequence(&mut sequence)
            .return_const(items.iter().collect::<Vec<_>>());
        host.expect_last_choice()
            .in_sequence(&mut sequence)
            .return_const(None);
        host.expect_pick()
            .in_sequence(&mut sequence)
            .returning(|_, _| Err(anyhow::anyhow!("picker failed")));
        let error = run(&host).unwrap_err();
        assert_eq!(error.to_string(), "picker failed");
    }

    #[test]
    fn no_editors_returns_install_guidance_before_picker_or_launch() {
        let mut host = MockOpenInHost::new();
        let mut sequence = Sequence::new();
        host.expect_workspace()
            .in_sequence(&mut sequence)
            .returning(|| Ok(workspace()));
        host.expect_editors()
            .in_sequence(&mut sequence)
            .return_const(Vec::new());
        let error = run(&host).unwrap_err();
        assert!(error.to_string().contains("no supported editor was found"));
    }
}
