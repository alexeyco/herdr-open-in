# plugin-registration Specification

## Purpose

Defines how the `alexeyco.open-in` plugin presents itself to herdr and how the single
binary decides what to do when herdr starts it. The manifest
(`herdr-plugin.toml`) is the registration contract: identity, target
platforms, build command, two pane declarations (`picker` as a popup,
`editor` as a tab), and one workspace-context action `alexeyco.open-in.open`. At
runtime the binary dispatches on environment variables herdr sets —
`HERDR_PLUGIN_ENTRYPOINT_ID` wins over `HERDR_PLUGIN_ACTION_ID` — selecting
one of the picker, editor, or action-pops-up-picker flows, or failing with a
named error. Every top-level failure follows one convention: a single stderr
line prefixed `open-in: ` and exit code 1.

## Requirements

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

### Requirement: Manifest declares picker and editor panes

The manifest SHALL declare two panes, both running
`./target/release/herdr-open-in`: a `picker` pane titled "Open in" placed as
a popup with width `40%` and height `16`, and an `editor` pane titled
"Editor" placed as a tab. The height is part of this contract: the popup's
inner area (height minus the two border rows herdr draws) must fit the full
11-entry catalog plus the frame chrome without scrolling.
(herdr-plugin.toml:12-24)

At height 16 the inner 14 lines are exactly 1 blank top line + 11 catalog
rows + 1 blank spacer + 1 help footer — the catalog fits at exact capacity
with no spare row; further catalog growth triggers the picker's scroll
window by design.

#### Scenario: Picker popup geometry

- **WHEN** herdr opens the `picker` pane
- **THEN** it renders a popup titled "Open in" sized 40% wide and 16 lines
  tall

#### Scenario: Full catalog fits without scrolling

- **WHEN** the popup opens on a machine with all catalog entries detected
- **THEN** all 11 editor rows, the blank top line, the blank spacer line, and
  the help footer occupy all 14 inner lines in the first frame with the scroll
  window at zero and no row hidden

#### Scenario: Editor pane placement

- **WHEN** herdr opens the `editor` pane
- **THEN** it is placed as a tab titled "Editor"

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
