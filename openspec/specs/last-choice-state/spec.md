# last-choice-state Specification

## Purpose

The last-choice state records which editor the user picked most recently, so
the next picker run can pre-highlight it. The choice lives as a bare editor id
plus a trailing newline in the file `last-editor` inside the per-plugin state
directory that herdr creates and exports through
`HERDR_PLUGIN_STATE_DIR` (src/state.rs:1-13). Reading is deliberately
forgiving — every missing or blank input yields "no last choice", never an
error — while writing is atomic via a sibling temp file and a rename. At the
call site in src/main.rs:123-127 persistence is best-effort: a save failure is
a warning on stderr and never blocks the launch the user just asked for.

## Requirements

### Requirement: State file location and format

The plugin SHALL keep the last chosen editor id in the file `last-editor`
directly under the directory named by `HERDR_PLUGIN_STATE_DIR`, written as the
bare id followed by a single trailing newline (src/state.rs:9-13, 70-82).

#### Scenario: Saved content

- **WHEN** the choice `zed` is saved to a state directory
- **THEN** the file `<state dir>/last-editor` contains exactly `zed\n`

#### Scenario: Round trip

- **WHEN** an id is saved and then read back from the same directory
- **THEN** the read returns that id

### Requirement: Forgiving read

Reading the last choice SHALL never return an error. It SHALL return "no last
choice" when `HERDR_PLUGIN_STATE_DIR` is unset or blank (after trimming), when
the `last-editor` file is missing or unreadable, or when its contents are only
whitespace; otherwise it SHALL return the file contents with surrounding
whitespace trimmed (src/state.rs:15-21, 31-39, 58-68; tests
src/state.rs:122-157).

#### Scenario: Unset state dir reads as no choice

- **WHEN** `HERDR_PLUGIN_STATE_DIR` is unset or contains only whitespace and
  the last choice is read
- **THEN** the read yields no choice and no error is raised

#### Scenario: Missing file reads as no choice

- **WHEN** the state directory exists but has no `last-editor` file
- **THEN** the read yields no choice

#### Scenario: Whitespace around the id is trimmed

- **WHEN** the file contains an id surrounded by spaces or newlines
- **THEN** the read yields the bare id

### Requirement: Atomic save

Saving SHALL create the state directory when missing, write the id to a
sibling temp file named `last-editor.<pid>.tmp` in that directory, and rename
the temp file over `last-editor`, so a concurrent reader never observes a
half-written file; if the rename fails the temp file SHALL be removed and an
error returned (src/state.rs:70-82).

#### Scenario: Missing directory is created

- **WHEN** a choice is saved into a state directory that does not exist yet
- **THEN** the directory is created and the `last-editor` file ends up holding
  the id

#### Scenario: Failed rename leaves no temp file

- **WHEN** the rename step fails
- **THEN** an error is returned and the `last-editor.<pid>.tmp` file is
  removed

### Requirement: Save errors name the environment variable

Saving SHALL fail with an error naming `HERDR_PLUGIN_STATE_DIR` when that
variable is unset or blank; reading in the same situation is not an error
(src/state.rs:47-56; tests src/state.rs:160-174).

#### Scenario: Save without a state dir

- **WHEN** `HERDR_PLUGIN_STATE_DIR` is unset and the plugin tries to save a
  choice
- **THEN** the save fails with an error message that mentions
  `HERDR_PLUGIN_STATE_DIR`

### Requirement: Best-effort persistence at the call site

After the user selects an editor, the picker flow SHALL save its id before
launching, and a save failure SHALL NOT abort the flow: the plugin prints
`open-in: warning: could not save the last editor: <error>` on stderr and
launches the chosen editor anyway; the process result is then determined by
the launch alone, unaffected by the save failure (src/main.rs:123-129).

#### Scenario: Save failure warns but launches

- **WHEN** saving the chosen id fails (for example the state directory cannot
  be created)
- **THEN** the warning line is printed on stderr, the editor is launched, and
  the process result is unaffected by the save failure

### Requirement: Saved id drives the picker preselect

The picker flow SHALL read the last choice before showing the dialog and pass
its id as the picker's preselect argument (src/main.rs:117-118). The stored id
SHALL NOT be validated against the currently detected editors at read or save
time — whatever is on disk is handed to the picker unchanged. How the picker
interprets a missing or unknown preselect (its first-row fallback) is owned by
the editor-picker capability, not restated here.

#### Scenario: Previously chosen editor is preselected

- **WHEN** the last choice holds the id of an editor that is currently
  detected
- **THEN** the picker opens with that row highlighted

#### Scenario: Stale id is passed through unvalidated

- **WHEN** the last choice holds an id that is not among the detected editors
- **THEN** the picker flow still passes that id as the preselect argument
  without filtering it, leaving the choice of highlighted row to the
  editor-picker fallback rules
