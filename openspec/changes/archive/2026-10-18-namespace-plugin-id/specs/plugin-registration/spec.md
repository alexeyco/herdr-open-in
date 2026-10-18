# plugin-registration — delta for namespace-plugin-id

## MODIFIED Requirements

### Requirement: Manifest declares plugin identity and build

The plugin SHALL register with herdr through `herdr-plugin.toml` declaring
id `alexeyco.open-in`, name "Open in", version `0.4.0`, minimum herdr
version `0.9.0`, platforms `linux` and `macos`, and a build command of
`cargo build --release --locked`. (herdr-plugin.toml:2-10)

#### Scenario: Manifest fields are readable by herdr

- **WHEN** herdr loads the plugin manifest
- **THEN** the plugin is identified as `alexeyco.open-in` at version 0.4.0,
  is built with `cargo build --release --locked`, and is offered only on
  linux and macos

#### Scenario: Version stays in sync with the crate

- **WHEN** the plugin version is bumped
- **THEN** the manifest and the crate declare the same version string, and
  herdr reports the plugin at that version (both `herdr-plugin.toml:4` and
  `Cargo.toml:3` currently say `0.4.0`, and `Cargo.lock` carries the same
  version for the `herdr-open-in` package)

### Requirement: Manifest declares the open action

The manifest SHALL declare one action with the local id `open` — kept
dot-free because herdr forbids dots in local action ids — titled "Open in
editor" with contexts `["workspace"]`, running
`./target/release/herdr-open-in`; herdr qualifies it to
`alexeyco.open-in.open` (plugin id + `.` + action id).
(herdr-plugin.toml:26-31)

#### Scenario: Action is available in workspace context

- **WHEN** herdr lists actions for a workspace
- **THEN** "Open in editor" from plugin `alexeyco.open-in` is offered, and
  invoking it by the qualified id `alexeyco.open-in.open` runs the action

### Requirement: Entrypoint variable wins over action variable in dispatch

The binary SHALL choose its mode from `HERDR_PLUGIN_ENTRYPOINT_ID` and
`HERDR_PLUGIN_ACTION_ID`: entrypoint `picker` runs the picker dialog,
entrypoint `editor` runs the terminal-editor flow, and when no entrypoint is
set but the action id is set the binary opens the picker popup by running
the herdr binary at `$HERDR_BIN_PATH`. herdr passes the bare local action
id (`open`) in `HERDR_PLUGIN_ACTION_ID`, never the qualified
`alexeyco.open-in.open`, and dispatch checks only whether the variable is
set, not its value. Environment values that are unset, non-UTF-8, or blank
after trimming count as absent. (main.rs:29-47, 71-101, 209-214)

#### Scenario: Entrypoint present alongside action id

- **WHEN** both `HERDR_PLUGIN_ENTRYPOINT_ID=picker` and
  `HERDR_PLUGIN_ACTION_ID=open` are set
- **THEN** the process runs the picker dialog, not the action flow
  (verified by the `mode_from` tests, main.rs:226-229)

#### Scenario: Action invocation opens the picker popup

- **WHEN** only the action id is set
- **THEN** the binary invokes `$HERDR_BIN_PATH plugin pane open` for the
  `picker` entrypoint as a focused popup and exits when herdr accepts

### Requirement: Unknown entrypoint and unbound invocation fail with named errors

The binary SHALL exit with an error naming the problem when
`HERDR_PLUGIN_ENTRYPOINT_ID` holds a value other than `picker` or `editor`
(error text of the form "unknown plugin entrypoint `<id>`; expected
`picker` or `editor`"), and when neither the entrypoint nor the action id is
set (error text stating that neither `HERDR_PLUGIN_ENTRYPOINT_ID` nor
`HERDR_PLUGIN_ACTION_ID` is set and directing the user to bind/invoke
`alexeyco.open-in.open` through herdr). (main.rs:78-85)

#### Scenario: Bogus entrypoint string

- **WHEN** `HERDR_PLUGIN_ENTRYPOINT_ID` is set to any value other than
  `picker` or `editor`, with or without an action id
- **THEN** the process prints the unknown-entrypoint error including the
  given id and exits 1

#### Scenario: Run directly outside herdr

- **WHEN** the binary is executed with neither environment variable set
- **THEN** it exits 1 with the unbound-invocation error naming
  `alexeyco.open-in.open` rather than showing any UI
