# Tasks: Focus terminal editor tabs

## 1. Request focus for terminal tabs

- [x] 1.1 Restore the baseline terminal-tab open argv without `--focus`, parse `/result/plugin_pane/pane/tab_id` alongside `/result/plugin_pane/pane/pane_id`, then invoke `tab focus <TAB_ID>` in `src/tab.rs`; test exact argv, missing/blank tab-id errors without a focus call, and focus success/failure handling (scenarios: "argv shape", "Acknowledged tab", "Success exit without pane id", "Success exit without tab id", "Focus acknowledged tab", "Focus command rejected", "Focus after acknowledged terminal tab"). Keep the picker popup's `--focus` unchanged.
- [x] 1.2 Preserve the directory-with-spaces argv behavior and verify coverage still asserts `--cwd` and `OPEN_IN_DIR` values travel verbatim (scenario: "Directory containing spaces").

## 2. Verify the change

- [x] 2.1 Update `docs/how-it-works.md` to document the terminal-tab open request followed by `$HERDR_BIN_PATH tab focus <TAB_ID>`. Leave GUI editor behavior and picker popup behavior unchanged.
- [x] 2.2 With a linked plugin and a herdr workspace, select `nvim`, `vim`, and `helix` in turn; for each, confirm the new editor tab becomes active and accepts keyboard input (scenario: "Terminal editor tab receives focus").
- [x] 2.3 Run the project checks in `docs/development.md` and confirm the tab-focus command receives the acknowledged tab id while the terminal-tab open argv excludes `--focus` and the picker popup argv retains its existing `--focus` (scenarios: "argv shape", "Focus acknowledged tab", "Directory containing spaces").
- [x] 2.4 Run `openspec validate focus-terminal-editor`; require VALID (format validation only, not archive-ability).
