# Design: Focus terminal editor tabs

Restore and preserve the baseline terminal-tab pane-open argv exactly: `$HERDR_BIN_PATH plugin pane open --plugin alexeyco.open-in --entrypoint editor --placement tab --workspace <workspace_id> --cwd <dir> --env OPEN_IN_EDITOR=<editor_id> --env OPEN_IN_DIR=<dir>`. This request MUST NOT include `--focus`; the user verified that option leaves focus on the agent tab. After the open response passes its existing error check and contains a non-blank pane id at `/result/plugin_pane/pane/pane_id`, parse the non-blank tab id at `/result/plugin_pane/pane/tab_id`. Invoke `$HERDR_BIN_PATH tab focus <TAB_ID>` as exact argv elements without a shell. Treat exit status zero as success. On a non-zero exit, report the herdr binary path, exit status, and trimmed stderr. A missing or blank tab id is an error naming the JSON path and raw stdout; do not run the focus command. The existing picker popup argv retains its own `--focus`; GUI editors do not use this terminal-tab path.

Both herdr calls remain argv-exec invocations. The open response's tab id becomes the argument to the tab-focus call. Editor environment values and working-directory values do not change.

Files to touch:

- `src/tab.rs` — parse the acknowledged tab id, invoke the tab-focus command, and test its argv and success/failure handling.
- `docs/how-it-works.md` — document the terminal-tab open request followed by `tab focus <TAB_ID>`.
- `openspec/specs/herdr-pane-control/spec.md` — when synchronizing the change, merge the modified pane-success requirement and added tab-focus requirement; this file is outside the change artifact and is not edited during planning.

Acceptance verification requires a linked plugin, a herdr workspace, and the three terminal editors. Invoke the picker, select each editor in turn, and observe that the corresponding newly opened tab becomes active and accepts keyboard input. The manual check validates herdr's runtime focus behavior in addition to the argv unit test.
