# terminal-editor-exec

## Purpose

When the user picks a terminal editor, the picker asks herdr to open a new
`tab`-placement pane running this plugin's own binary with
`HERDR_PLUGIN_ENTRYPOINT_ID=editor`, passing the chosen editor id and the
resolved directory as `OPEN_IN_EDITOR` and `OPEN_IN_DIR`. This capability is
what that pane does: it reads the two environment variables, resolves the
editor id against the catalog, and runs the editor's CLI in the foreground —
stdio inherited so the editor owns the tab's tty — waiting for it to exit and
mapping any failure to a single stderr line and exit status 1. It is the
terminal counterpart of the detached GUI launch, and unlike GUI launch it does
no alias or absolute-path probing: it executes the row's bare primary command.

## Requirements

### Requirement: Mode requires both environment variables

The `editor` entrypoint SHALL read `OPEN_IN_EDITOR` and `OPEN_IN_DIR` as
required values; a variable that is unset, non-UTF-8, or blank (whitespace-only
after trimming) is treated as absent and fails with an error naming the
variable.

#### Scenario: Missing editor id

- **WHEN** the pane runs with `HERDR_PLUGIN_ENTRYPOINT_ID=editor` and
  `OPEN_IN_EDITOR` unset or blank
- **THEN** the process prints, under the `open-in: ` prefix, an error naming
  the variable, of the form `OPEN_IN_EDITOR is not set or blank`, on stderr
  and exits 1 before consulting the directory
  (`src/main.rs:142-144,207-219`, prefix and exit at `src/main.rs:49-53`)

#### Scenario: Missing directory

- **WHEN** `OPEN_IN_EDITOR` is set but `OPEN_IN_DIR` is unset or blank
- **THEN** the error names `OPEN_IN_DIR` in the same form
  (`src/main.rs:144,216-219`)

### Requirement: Editor id resolves against the full catalog

The editor id SHALL be resolved by exact match against the entire catalog with
no kind filter, and an unknown id SHALL fail with an error naming the id and
listing every known catalog id.

#### Scenario: Known terminal id resolves

- **WHEN** `OPEN_IN_EDITOR` is a catalog id such as `nvim`, `vim`, or `helix`
- **THEN** the corresponding row is selected for execution
  (`src/main.rs:145,159-171`)

#### Scenario: Unknown id lists the known ones

- **WHEN** `OPEN_IN_EDITOR` holds an id that is not in the catalog
- **THEN** the process fails with `open-in: unknown editor \`<id>\`; known
  editors: <comma-separated catalog ids>` and exits 1 (`src/main.rs:159-165`,
`src/main.rs:173-180`)

#### Scenario: GUI id is not filtered out

- **WHEN** a GUI row's id (for example `code`) arrives in `OPEN_IN_EDITOR`
- **THEN** the resolution still succeeds and that row's command is executed in
  the foreground — this path applies no kind filter (verified: the lookup at
  `src/main.rs:159-171` searches `editors::CATALOG` without checking
  `EditorKind`; normal picker flow only sends terminal ids here via
  `src/main.rs:133-137` and `src/tab.rs:76-96`)

### Requirement: Foreground execution owning the tty

The chosen editor SHALL run as `Command::new(<row's bare cmd>)` with the row's
argv template and the directory substituted for the single `<dir>` placeholder,
with default (inherited) stdio so the editor owns the tab's tty, and the
plugin SHALL wait for the child's exit status before returning.

#### Scenario: Terminal row argv

- **WHEN** `OPEN_IN_EDITOR=nvim` and `OPEN_IN_DIR=/home/u/demo`
- **THEN** the process runs `nvim /home/u/demo` (the bare `<dir>` template
  substituted) and blocks until nvim exits (`src/main.rs:147-149,183-195`,
  template at `src/editors.rs:135-144`, test at `src/main.rs:277-284`)

#### Scenario: Bare command only, no alias probing

- **WHEN** the editor row's primary command is not on PATH but an alias or a
  known absolute install location would resolve it
- **THEN** this path still executes the bare `editor.cmd` name — unlike GUI
  launch's `find_cli`, no alias or fallback-path probing happens here
  (verified: `src/main.rs:147` uses `editor.cmd` directly, while
  `src/launch.rs:28-55` is the consumer of `probe_names` and fallbacks)

### Requirement: Failure reporting

A spawn failure SHALL produce an error naming the editor's display name and
the command with the OS error appended; a non-zero child exit SHALL produce an
error naming the display name and the exit status. Either error goes to stderr
under the `open-in: ` prefix and the process exits 1; a zero child exit is a
clean success with no output.

#### Scenario: Command cannot be spawned

- **WHEN** the row's bare command cannot be executed (for example it is
  missing from PATH)
- **THEN** the process prints
  `open-in: failed to launch <Name> (\`<cmd>\`): <os error>` and exits 1
(`src/main.rs:147-150,49-53`)

#### Scenario: Editor exits non-zero

- **WHEN** the editor process runs but exits with a non-zero status
- **THEN** the process prints `open-in: <Name> exited with <status>` and exits
  1 (`src/main.rs:151-155,49-53`)

#### Scenario: Editor exits zero

- **WHEN** the editor process exits successfully
- **THEN** the plugin prints nothing to stderr and exits 0
  (`src/main.rs:151-153`)
