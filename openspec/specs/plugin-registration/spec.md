# plugin-registration Specification

## Purpose

Defines how the open-in plugin presents itself to herdr and how the single
binary decides what to do when herdr starts it. The manifest
(`herdr-plugin.toml`) is the registration contract: identity, target
platforms, build command, two pane declarations (`picker` as a popup,
`editor` as a tab), and one workspace-context action `open-in.open`. At
runtime the binary dispatches on environment variables herdr sets —
`HERDR_PLUGIN_ENTRYPOINT_ID` wins over `HERDR_PLUGIN_ACTION_ID` — selecting
one of the picker, editor, or action-pops-up-picker flows, or failing with a
named error. Every top-level failure follows one convention: a single stderr
line prefixed `open-in: ` and exit code 1.

## Requirements

### Requirement: Manifest declares plugin identity and build

The plugin SHALL register with herdr through `herdr-plugin.toml` declaring
id `open-in`, name "Open in", version `0.2.0`, minimum herdr version `0.9.0`,
platforms `linux` and `macos`, and a build command of
`cargo build --release --locked`. (herdr-plugin.toml:2-10)

#### Scenario: Manifest fields are readable by herdr

- **WHEN** herdr loads the plugin manifest
- **THEN** the plugin is identified as `open-in` at version 0.2.0, is built
  with `cargo build --release --locked`, and is offered only on linux and
  macos

#### Scenario: Version stays in sync with the crate

- **WHEN** the plugin version is bumped
- **THEN** the manifest and the crate declare the same version string, and
  herdr reports the plugin at that version (both `herdr-plugin.toml:4` and
  `Cargo.toml:3` currently say `0.2.0`)

### Requirement: Manifest declares picker and editor panes

The manifest SHALL declare two panes, both running
`./target/release/herdr-open-in`: a `picker` pane titled "Open in" placed as
a popup with width `40%` and height `14`, and an `editor` pane titled
"Editor" placed as a tab. (herdr-plugin.toml:12-24; the popup geometry is
part of this contract by decision)

#### Scenario: Picker popup geometry

- **WHEN** herdr opens the `picker` pane
- **THEN** it renders a popup titled "Open in" sized 40% wide and 14 lines
  tall

#### Scenario: Editor pane placement

- **WHEN** herdr opens the `editor` pane
- **THEN** it is placed as a tab titled "Editor"

### Requirement: Manifest declares the open action

The manifest SHALL declare one action `open` (invoked as `open-in.open`)
titled "Open in editor" with contexts `["workspace"]`, running
`./target/release/herdr-open-in`. (herdr-plugin.toml:26-31)

#### Scenario: Action is available in workspace context

- **WHEN** herdr lists actions for a workspace
- **THEN** "Open in editor" from plugin `open-in` is offered

### Requirement: Entrypoint variable wins over action variable in dispatch

The binary SHALL choose its mode from `HERDR_PLUGIN_ENTRYPOINT_ID` and
`HERDR_PLUGIN_ACTION_ID`: entrypoint `picker` runs the picker dialog,
entrypoint `editor` runs the terminal-editor flow, and when no entrypoint is
set but the action id is set the binary opens the picker popup by running
the herdr binary at `$HERDR_BIN_PATH`. Environment values that are unset,
non-UTF-8, or blank after trimming count as absent. (main.rs:29-47, 71-101,
209-214)

#### Scenario: Entrypoint present alongside action id

- **WHEN** both `HERDR_PLUGIN_ENTRYPOINT_ID=picker` and
  `HERDR_PLUGIN_ACTION_ID=open-in.open` are set
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
`open-in.open` through herdr). (main.rs:78-85)

#### Scenario: Bogus entrypoint string

- **WHEN** `HERDR_PLUGIN_ENTRYPOINT_ID` is set to any value other than
  `picker` or `editor`, with or without an action id
- **THEN** the process prints the unknown-entrypoint error including the
  given id and exits 1

#### Scenario: Run directly outside herdr

- **WHEN** the binary is executed with neither environment variable set
- **THEN** it exits 1 with the unbound-invocation error rather than showing
  any UI

### Requirement: Error and exit convention

Every top-level failure SHALL print exactly one stderr line prefixed
`open-in: ` and exit with code 1; a clean cancel of the picker (Escape or
`q`) SHALL exit 0 without output. The `open-in: ` prefix and the exit codes
are the stable contract; exact message wording may change. (main.rs:50-53,
115-121)

#### Scenario: Any flow error

- **WHEN** any mode fails for any reason
- **THEN** stderr carries one line beginning `open-in: ` and the process
  exits 1

#### Scenario: User cancels the picker

- **WHEN** the picker dialog is dismissed without a choice
- **THEN** the process exits 0 silently
