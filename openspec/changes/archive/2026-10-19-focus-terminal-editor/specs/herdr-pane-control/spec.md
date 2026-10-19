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

#### Scenario: Terminal editor tab receives focus

- **WHEN** the plugin opens a tab for `nvim`, `vim`, or `helix`
- **THEN** the newly opened terminal editor tab is active and receives
  keyboard input focus

### Requirement: Tab success requires a pane id

For the terminal-tab call the plugin SHALL parse herdr's stdout JSON and
require a non-blank string at `/result/plugin_pane/pane/pane_id`. On a zero
exit the parsed response's `error` field is checked before the pane id
(tab.rs:142-144): an `error` in the response is reported with herdr's own
wording even when the exit status said success. Only when the response
carries no error does a missing or blank pane id produce an error naming the
missing JSON path and embedding the raw stdout, and malformed JSON an error
naming the response as invalid and embedding the serde message (of the forms
"herdr response has no result.plugin_pane.pane.pane_id: <stdout>" and
"invalid herdr response: <serde error>"). After a valid pane id is
acknowledged, the plugin SHALL require a non-blank string at
`/result/plugin_pane/pane/tab_id`, alongside the pane id. A missing or blank tab id SHALL produce an
error naming the JSON path and embedding raw stdout. The plugin SHALL invoke
`$HERDR_BIN_PATH tab focus <tab_id>` and SHALL return success only after that
command exits zero. On a non-zero exit, the plugin SHALL report the herdr
binary path, exit status, and trimmed stderr.
(tab.rs:139-156)

#### Scenario: Acknowledged tab

- **WHEN** stdout is
  `{"result":{"plugin_pane":{"pane":{"pane_id":"pane-42","tab_id":"tab-1"}}}}`-shaped JSON
  and `herdr tab focus tab-1` exits zero
- **THEN** the plugin accepts the pane and tab ids, focuses `tab-1`, and
  returns success

#### Scenario: Success exit without pane id

- **WHEN** herdr exits 0 with JSON lacking the pane id path or with a
  whitespace-only id, and the response carries no `error` field
- **THEN** the plugin reports an error naming the pane-id JSON path and the
  raw stdout without invoking the focus command

#### Scenario: Success exit without tab id

- **WHEN** herdr exits 0 with a valid pane id but JSON lacks
  `/result/plugin_pane/pane/tab_id` or contains a whitespace-only tab id
- **THEN** the plugin reports an error naming the tab-id JSON path and raw
  stdout without invoking the tab-focus command

## ADDED Requirements

### Requirement: Terminal-tab focus command argv

After the terminal-tab pane-open response contains a valid tab id, the plugin
SHALL execute `$HERDR_BIN_PATH tab focus <tab_id>` as exact argv elements,
without a shell. Exit zero SHALL complete successfully. A non-zero exit
SHALL report the herdr binary path, status, and trimmed stderr. The
terminal-tab pane-open request SHALL omit `--focus`; the picker popup keeps
its existing `--focus` option.

#### Scenario: Focus acknowledged tab

- **WHEN** the pane-open response contains tab id `tab-1` and herdr exits
  zero for `tab focus tab-1`
- **THEN** the plugin completes successfully after invoking the focus command
  with `tab-1` as one argv element

#### Scenario: Focus command rejected

- **WHEN** herdr exits non-zero for `tab focus tab-1`
- **THEN** the plugin reports the herdr binary path, exit status, and
  trimmed stderr

#### Scenario: Focus after acknowledged terminal tab

- **WHEN** herdr acknowledges a terminal tab with valid pane and tab ids
- **THEN** the plugin invokes `tab focus <tab_id>` after parsing the open
  response and before the action returns success
