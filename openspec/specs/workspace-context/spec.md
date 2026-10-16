# workspace-context Specification

## Purpose

Defines how the plugin turns the JSON herdr passes in
`HERDR_PLUGIN_CONTEXT_JSON` into the two facts it needs: the directory to
open and the workspace to open a tab in. The directory is resolved along a
fixed precedence chain — `worktree.checkout_path`, then `workspace_cwd`,
then `focused_pane_cwd` — where non-string or whitespace-only values are
treated as absent and skipped. The `workspace_id` is read from the same
document and is mandatory. Parsing is done with serde_json, so JSON string
escapes (backslashes, unicode) are decoded into real path characters.
Missing, blank, or malformed input produces errors that name the variable or
the JSON problem rather than falling back to a default.

## Requirements

### Requirement: Picker mode requires the context variable

The picker flow SHALL require `HERDR_PLUGIN_CONTEXT_JSON`; a value that is
unset or blank (after trimming) is an error naming the variable
("HERDR_PLUGIN_CONTEXT_JSON is not set or blank"). (main.rs:106, 34-35,
217-219)

#### Scenario: Context variable absent

- **WHEN** the picker entrypoint runs without `HERDR_PLUGIN_CONTEXT_JSON`
  set, or with it set to whitespace only
- **THEN** the process fails with the error naming the variable and exits
  under the `open-in: ` / exit-1 convention

### Requirement: Directory resolution chain

The plugin SHALL resolve the open directory as the first present, non-blank
string among `worktree.checkout_path`, `workspace_cwd`, and
`focused_pane_cwd`, in that order, and SHALL yield that value trimmed
(leading and trailing whitespace removed; context.rs:11-13, 33); a non-string
or whitespace-only value counts as absent and the chain falls through. When no
candidate yields a value the plugin SHALL report "no directory in plugin
context; invoke this action inside a workspace". (context.rs:5-34)

#### Scenario: Checkout path wins

- **WHEN** the context JSON carries a non-blank `worktree.checkout_path`
- **THEN** that value is used even when `workspace_cwd` and
  `focused_pane_cwd` are also present

#### Scenario: Fallback and fall-through

- **WHEN** `worktree.checkout_path` is missing, whitespace-only, or not a
  string
- **THEN** `workspace_cwd` is used, and failing that `focused_pane_cwd`

#### Scenario: No usable candidate

- **WHEN** all three candidates are absent or blank
- **THEN** the plugin reports the no-directory error and does not open
  anything

### Requirement: Workspace id is mandatory

The plugin SHALL read `workspace_id` as a string from the same context JSON;
a missing, non-string, or whitespace-only value is reported as
"no workspace_id in plugin context; invoke this action inside a workspace".
(context.rs:38-50)

#### Scenario: Valid workspace id

- **WHEN** the context JSON contains a non-blank string `workspace_id`
- **THEN** that value (trimmed) is returned for use as the tab target

#### Scenario: Invalid workspace id

- **WHEN** `workspace_id` is absent, a number, or whitespace only
- **THEN** the plugin reports the no-workspace-id error

### Requirement: Context JSON parsing

The plugin SHALL parse the context with serde_json and report malformed
input as "invalid plugin context JSON: <serde error>"; valid JSON string
escapes SHALL be decoded, including backslashes and unicode, so paths like
`C:\\Users\\caf\u00e9` resolve to the intended filesystem path.
(context.rs:9-10, 39-40, 108-111)

#### Scenario: Malformed JSON

- **WHEN** `HERDR_PLUGIN_CONTEXT_JSON` does not contain valid JSON
- **THEN** the plugin fails with the invalid-JSON error embedding the serde
  message

#### Scenario: Escaped path decoding

- **WHEN** the directory value in the JSON contains escape sequences
- **THEN** the resolved directory is the unescaped string as the filesystem
  sees it
