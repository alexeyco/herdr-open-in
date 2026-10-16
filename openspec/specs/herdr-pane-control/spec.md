# herdr-pane-control Specification

## Purpose

Defines how the plugin calls back into herdr to ask for panes, and where
that responsibility ends: once herdr has acknowledged the request, the pane
itself belongs to herdr. The herdr binary is addressed only through
`$HERDR_BIN_PATH` (never a bare `herdr` on PATH), and requests are made with
exact argv arrays executed without a shell, so values containing spaces
travel unquoted. Two calls exist: opening a tab that runs the plugin's
`editor` entrypoint with the chosen editor and directory passed as the
custom variables `OPEN_IN_EDITOR`/`OPEN_IN_DIR` (custom names because herdr
forbids `--env` from overriding its managed `HERDR_*` variables), and
opening the `picker` popup. Tab success additionally requires a non-blank
pane id in herdr's JSON response; failures surface herdr's own error wording
when the response carries one.

## Requirements

### Requirement: Herdr binary resolved only from HERDR_BIN_PATH

The plugin SHALL resolve the herdr binary exclusively from
`HERDR_BIN_PATH`, trimming the value; unset, empty, or whitespace-only is an
error naming the variable and the remedy, of the form `HERDR_BIN_PATH is not
set or blank; invoke this action from herdr`. A bare `herdr` command is never
assumed.
(tab.rs:17-19, 131)

#### Scenario: Path provided with surrounding whitespace

- **WHEN** `HERDR_BIN_PATH` is set with padding whitespace
- **THEN** the trimmed value is used as the executable path

#### Scenario: Path missing

- **WHEN** any pane-open call is attempted without a usable
  `HERDR_BIN_PATH`
- **THEN** the plugin fails with the error naming the variable

### Requirement: Terminal-tab pane open argv

The plugin SHALL open a terminal editor by executing
`$HERDR_BIN_PATH plugin pane open --plugin open-in --entrypoint editor
--placement tab --workspace <workspace_id> --cwd <dir>
--env OPEN_IN_EDITOR=<editor_id> --env OPEN_IN_DIR=<dir>` as an exact argv
sequence, each value its own element (argv-exec, no shell, so directories
with spaces pass unquoted). The `OPEN_IN_*` names are custom because `--env`
cannot override herdr-managed `HERDR_*` variables. (tab.rs:5-9, 76-96;
tested at tab.rs:188-211, 215-224)

#### Scenario: argv shape

- **WHEN** a terminal editor is chosen for workspace `ws-1` on
  `/home/u/demo`
- **THEN** the spawned argv is exactly
  `plugin pane open --plugin open-in --entrypoint editor --placement tab
--workspace ws-1 --cwd /home/u/demo --env OPEN_IN_EDITOR=<id>
--env OPEN_IN_DIR=/home/u/demo` in that order

#### Scenario: Directory containing spaces

- **WHEN** the resolved directory is `/tmp/x y`
- **THEN** it travels as one argv element and both the `--cwd` value and the
  `OPEN_IN_DIR=/tmp/x y` env value carry it verbatim, with no shell quoting

### Requirement: Picker popup pane open argv

The plugin SHALL open the picker by executing
`$HERDR_BIN_PATH plugin pane open --plugin open-in --entrypoint picker
--placement popup --focus`, and SHALL treat the call as successful on any
zero exit status: a popup response carries no pane id, so only the exit
status is checked. (tab.rs:42-48, 101-117; tested at tab.rs:242-258)

#### Scenario: Popup request accepted

- **WHEN** herdr exits 0 after the popup request
- **THEN** the action mode completes without parsing a pane id from stdout

#### Scenario: Second popup while one is open

- **WHEN** herdr rejects the popup request (e.g. `ui_busy` because another
  popup is already open)
- **THEN** the plugin surfaces herdr's own error text through the failure
  formatting below (per doc at tab.rs:44-45)

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
"invalid herdr response: <serde error>"). Once a pane id is acknowledged the
plugin's responsibility for the pane ends. (tab.rs:139-156)

#### Scenario: Acknowledged tab

- **WHEN** stdout is
  `{"result":{"plugin_pane":{"pane":{"pane_id":"pane-42"}}}}`-shaped JSON
- **THEN** the plugin accepts `pane-42` and returns success without further
  interaction with the pane

#### Scenario: Success exit without pane id

- **WHEN** herdr exits 0 with JSON lacking the pane id path or with a
  whitespace-only id, and the response carries no `error` field
- **THEN** the plugin reports an error naming the pane-id JSON path and the
  raw stdout

### Requirement: Failure formatting from herdr responses

On a non-zero exit the plugin SHALL format the failure from herdr's own
output: an `error` object with `code` and `message` becomes
"herdr reported error <code>: <message>" (code only when the message is
absent; `unknown` when the code is absent); a bare `error` string becomes
"herdr reported error: <text>"; otherwise the status and trimmed stderr
become "<bin> plugin pane open exited with <status>: <stderr>". A failure to
spawn herdr at all is "failed to run <bin>: <err>". (tab.rs:52-71, 164-181)

#### Scenario: Structured error response

- **WHEN** herdr exits non-zero with stdout carrying
  `{"error":{"code":"plugin_not_found","message":"no such plugin"}}`
- **THEN** the reported message names the code and herdr's message

#### Scenario: Error object without message

- **WHEN** the `error` object has a `code` but no usable `message`
- **THEN** the reported message states the code alone, and a missing code is
  reported as `unknown`

#### Scenario: Unparseable failure output

- **WHEN** herdr exits non-zero and stdout is not JSON with an `error` key
- **THEN** the message carries the binary path, exit status, and trimmed
  stderr
