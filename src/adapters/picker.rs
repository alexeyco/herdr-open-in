//! Compact editor picker implemented with ratatui and crossterm.

use crate::adapters::editors::Editor;
use ratatui::crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    widgets::{List, ListItem, ListState},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectionAction {
    Up,
    Down,
    Select,
    Cancel,
    Ignore,
}

fn update_selection(selected: usize, count: usize, action: SelectionAction) -> usize {
    match action {
        SelectionAction::Up => selected.saturating_sub(1),
        SelectionAction::Down => selected.saturating_add(1).min(count.saturating_sub(1)),
        _ => selected,
    }
}

fn action_for_key(key: KeyCode) -> SelectionAction {
    match key {
        KeyCode::Up => SelectionAction::Up,
        KeyCode::Down => SelectionAction::Down,
        KeyCode::Enter => SelectionAction::Select,
        KeyCode::Esc => SelectionAction::Cancel,
        _ => SelectionAction::Ignore,
    }
}

fn selected_index(items: &[&'static Editor], preselect: Option<&str>) -> usize {
    preselect
        .and_then(|id| items.iter().position(|editor| editor.id == id))
        .unwrap_or(0)
}

fn run_events(
    items: &[&'static Editor],
    mut selected: usize,
    mut draw: impl FnMut(usize) -> Result<(), String>,
    mut read: impl FnMut() -> Result<KeyCode, String>,
) -> Result<Option<&'static Editor>, String> {
    loop {
        draw(selected)?;
        match action_for_key(read()?) {
            SelectionAction::Select => return Ok(Some(items[selected])),
            SelectionAction::Cancel => return Ok(None),
            action => selected = update_selection(selected, items.len(), action),
        }
    }
}

fn with_restore<T>(
    result: Result<T, String>,
    restore: impl FnOnce() -> Result<(), String>,
) -> Result<T, String> {
    match (result, restore()) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(format!("cannot restore terminal: {error}")),
        (Ok(value), Ok(())) => Ok(value),
    }
}

fn render(frame: &mut Frame<'_>, items: &[&'static Editor], selected: usize) {
    let [_, list_area, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, editor)| {
            let marker = if index == selected { "› " } else { "  " };
            ListItem::new(format!("{marker}{}", editor.name))
        })
        .collect::<Vec<_>>();
    let list = List::new(rows).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, list_area, &mut state);
}

/// Runs the picker and returns the chosen editor, or `None` when cancelled.
pub(crate) fn run(
    items: &[&'static Editor],
    preselect: Option<&str>,
) -> Result<Option<&'static Editor>, String> {
    if items.is_empty() {
        return Err("no editors to pick from".to_string());
    }

    {
        use ratatui::crossterm::{
            event::{self, Event},
            terminal::{disable_raw_mode, enable_raw_mode},
        };
        use ratatui::{Terminal, backend::CrosstermBackend};
        use std::io::stdout;

        let selected = selected_index(items, preselect);
        enable_raw_mode().map_err(|e| format!("cannot enable raw mode: {e}"))?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).map_err(|e| {
            let _ = disable_raw_mode();
            format!("cannot initialize editor picker: {e}")
        })?;
        let result = (|| -> Result<Option<&'static Editor>, String> {
            terminal
                .hide_cursor()
                .map_err(|e| format!("cannot hide cursor: {e}"))?;
            run_events(
                items,
                selected,
                |selected| {
                    terminal
                        .draw(|frame| render(frame, items, selected))
                        .map(|_| ())
                        .map_err(|e| format!("cannot draw editor picker: {e}"))
                },
                || loop {
                    if let Event::Key(key) =
                        event::read().map_err(|e| format!("cannot read picker input: {e}"))?
                    {
                        return Ok(key.code);
                    }
                },
            )
        })();

        with_restore(result, || {
            let cursor = terminal.show_cursor();
            let raw = disable_raw_mode();
            cursor
                .and(raw)
                .map_err(|e| format!("cannot restore terminal: {e}"))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::editors::CATALOG;
    use ratatui::{Terminal, backend::TestBackend};

    fn items() -> Vec<&'static Editor> {
        CATALOG.iter().take(3).collect()
    }

    #[test]
    fn selection_moves_within_bounds_and_ignores_other_actions() {
        assert_eq!(update_selection(0, 3, SelectionAction::Up), 0);
        assert_eq!(update_selection(0, 3, SelectionAction::Down), 1);
        assert_eq!(update_selection(2, 3, SelectionAction::Down), 2);
        assert_eq!(update_selection(1, 3, SelectionAction::Ignore), 1);
        assert_eq!(update_selection(1, 3, SelectionAction::Select), 1);
        assert_eq!(update_selection(1, 3, SelectionAction::Cancel), 1);
    }

    #[test]
    fn down_then_enter_selects_the_second_editor() {
        let editors = items();
        let mut keys = [KeyCode::Down, KeyCode::Enter].into_iter();
        let mut drawn = Vec::new();
        let result = run_events(
            &editors,
            selected_index(&editors, None),
            |selected| {
                drawn.push(selected);
                Ok(())
            },
            || Ok(keys.next().unwrap()),
        );
        assert_eq!(drawn, [0, 1]);
        assert_eq!(result.unwrap().map(|editor| editor.id), Some(editors[1].id));
    }

    #[test]
    fn escape_cancels_without_producing_a_selection() {
        let editors = items();
        let result = run_events(&editors, 0, |_| Ok(()), || Ok(KeyCode::Esc));
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn unsupported_key_codes_are_ignored_by_the_event_flow() {
        for key in [KeyCode::Char('j'), KeyCode::Char('k'), KeyCode::Char('q')] {
            assert_eq!(action_for_key(key), SelectionAction::Ignore);
        }
        let editors = items();
        let mut keys = [
            KeyCode::Char('j'),
            KeyCode::Char('k'),
            KeyCode::Char('q'),
            KeyCode::Enter,
        ]
        .into_iter();
        let mut drawn = Vec::new();
        let result = run_events(
            &editors,
            0,
            |selected| {
                drawn.push(selected);
                Ok(())
            },
            || Ok(keys.next().unwrap()),
        );
        assert_eq!(drawn, [0, 0, 0, 0]);
        assert_eq!(result.unwrap().map(|editor| editor.id), Some(editors[0].id));
    }

    #[test]
    fn initial_selection_uses_matching_id_and_defaults_to_first() {
        let editors = items();
        assert_eq!(selected_index(&editors, Some(editors[2].id)), 2);
        assert_eq!(selected_index(&editors, Some("unknown")), 0);
        assert_eq!(selected_index(&editors, None), 0);
    }

    #[test]
    fn terminal_restore_runs_after_selection_cancellation_and_failures() {
        let editors = items();
        for outcome in [
            run_events(&editors, 0, |_| Ok(()), || Ok(KeyCode::Enter)),
            run_events(&editors, 0, |_| Ok(()), || Ok(KeyCode::Esc)),
            run_events(
                &editors,
                0,
                |_| Err("draw failed".to_string()),
                || Ok(KeyCode::Enter),
            ),
            run_events(&editors, 0, |_| Ok(()), || Err("read failed".to_string())),
        ] {
            let mut restored = false;
            let _ = with_restore(outcome, || {
                restored = true;
                Ok(())
            });
            assert!(restored);
        }

        let mut restored = false;
        let setup = Err::<(), _>("setup failed".to_string());
        let result = with_restore(setup, || {
            restored = true;
            Ok(())
        });
        assert!(restored);
        assert!(matches!(result, Err(error) if error == "setup failed"));
    }

    #[test]
    fn empty_list_fails_before_terminal_rendering() {
        assert!(matches!(run(&[], None), Err(error) if error == "no editors to pick from"));
    }

    #[test]
    fn test_backend_renders_names_in_order_and_marks_selection() {
        let items = items();
        let backend = TestBackend::new(30, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, &items, 1)).unwrap();
        let buffer = terminal.backend().buffer();
        let text = (0..5)
            .map(|y| (0..30).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>();
        assert!(text[0].trim().is_empty());
        assert!(text[4].trim().is_empty());
        for (row, item) in text[1..4].iter().zip(items.iter()) {
            assert!(row.contains(item.name));
        }
        assert!(text[1].starts_with("  "));
        assert!(text[2].starts_with("› "));
        assert!(text[3].starts_with("  "));
        assert!(
            buffer[(0, 2)]
                .style()
                .add_modifier
                .contains(Modifier::REVERSED)
        );
        let rendered = text.join("\n");
        assert!(!rendered.contains('✓'));
        assert!(!rendered.contains("select"));
        assert!(!rendered.contains("cancel"));
    }

    #[test]
    fn long_list_keeps_selected_row_visible_without_extra_dialog_rows() {
        let items = CATALOG.iter().collect::<Vec<_>>();
        let height = 3;
        let backend = TestBackend::new(30, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &items, items.len() - 1))
            .unwrap();

        let buffer = terminal.backend().buffer();
        let visible = (0..height)
            .map(|y| (0..30).map(|x| buffer[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>();
        assert!(
            visible
                .iter()
                .any(|row| row.contains(items.last().unwrap().name))
        );
        assert!((0..height).any(|y| {
            buffer[(0, y)]
                .style()
                .add_modifier
                .contains(Modifier::REVERSED)
        }));
        let rendered = visible.join("\n");
        assert!(!rendered.contains('✓'));
        assert!(!rendered.contains("select"));
        assert!(!rendered.contains("cancel"));
    }
}
