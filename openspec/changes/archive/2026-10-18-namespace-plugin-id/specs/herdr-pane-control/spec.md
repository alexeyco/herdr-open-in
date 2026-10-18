# herdr-pane-control — delta for namespace-plugin-id

## MODIFIED Requirements

### Requirement: Terminal-tab pane open argv

The plugin SHALL open a terminal editor by executing
`$HERDR_BIN_PATH plugin pane open --plugin alexeyco.open-in --entrypoint
editor --placement tab --workspace <workspace_id> --cwd <dir>
--env OPEN_IN_EDITOR=<editor_id> --env OPEN_IN_DIR=<dir>` as an exact argv
sequence, each value its own element (argv-exec, no shell, so directories
with spaces pass unquoted). The `--plugin` value is the manifest plugin id
held in the `PLUGIN_ID` constant. The `OPEN_IN_*` names are custom because
`--env` cannot override herdr-managed `HERDR_*` variables.
(tab.rs:5-9, 21, 76-96; tested at tab.rs:188-211, 215-224)

#### Scenario: argv shape

- **WHEN** a terminal editor is chosen for workspace `ws-1` on
  `/home/u/demo`
- **THEN** the spawned argv is exactly
  `plugin pane open --plugin alexeyco.open-in --entrypoint editor
--placement tab --workspace ws-1 --cwd /home/u/demo
--env OPEN_IN_EDITOR=<id> --env OPEN_IN_DIR=/home/u/demo` in that order

#### Scenario: Directory containing spaces

- **WHEN** the resolved directory is `/tmp/x y`
- **THEN** it travels as one argv element and both the `--cwd` value and the
  `OPEN_IN_DIR=/tmp/x y` env value carry it verbatim, with no shell quoting

### Requirement: Picker popup pane open argv

The plugin SHALL open the picker by executing
`$HERDR_BIN_PATH plugin pane open --plugin alexeyco.open-in --entrypoint
picker --placement popup --focus`, and SHALL treat the call as successful
on any zero exit status: a popup response carries no pane id, so only the
exit status is checked. (tab.rs:42-48, 101-117; tested at tab.rs:242-258)

#### Scenario: Popup request accepted

- **WHEN** herdr exits 0 after the popup request
- **THEN** the action mode completes without parsing a pane id from stdout

#### Scenario: Second popup while one is open

- **WHEN** herdr rejects the popup request (e.g. `ui_busy` because another
  popup is already open)
- **THEN** the plugin surfaces herdr's own error text through the failure
  formatting below (per doc at tab.rs:44-45)
