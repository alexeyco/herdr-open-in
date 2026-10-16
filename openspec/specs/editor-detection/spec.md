# editor-detection

## Purpose

Editor detection decides which catalog rows are installed on the current
machine and therefore which rows the picker can offer. It is a pure filter
over the compile-time catalog (`src/editors.rs`): a row is installed when one
of its probe names (the primary CLI command, then each alias, in order) runs
successfully on PATH, or when its macOS app bundle is present and looks
complete. The result preserves catalog order — detection never sorts — and the
detected list is exactly what the picker renders, one row per editor. The same
probe-name list is shared with the GUI launcher's CLI lookup so detection and
launch always agree on which names identify a row. When nothing is installed,
the picker flow fails with a single descriptive error before any dialog opens.

## Requirements

### Requirement: Installed via CLI probe

An editor SHALL count as installed when any of its probe names — the primary
command first, then each alias in order — is found on PATH, tested by running
`<name> --version` with both output streams discarded and treating exit status
0 as installed.

#### Scenario: Primary command answers

- **WHEN** `code --version` exits 0 on the machine
- **THEN** the `code` row is among the detected editors and its output is not
  shown to the user (`src/editors.rs:197-206`)

#### Scenario: Alias rescues a row

- **WHEN** no `zed` command exists but `zeditor --version` exits 0
- **THEN** the `zed` row is still detected through its alias
  (`src/editors.rs:176-182,191-194`, test at `src/editors.rs:323-341`)

#### Scenario: Non-zero exit is not installed

- **WHEN** every probe name for a row either is missing from PATH or exits
  non-zero, and no bundle check passes for the row
- **THEN** the row is not detected (`src/editors.rs:199-206` — a missing
  command makes `status()` fail; success is required)

### Requirement: Installed via macOS app bundle

On macOS a row with a bundle stem SHALL additionally count as installed when
`<stem>.app` is a directory under `/Applications` or `$HOME/Applications` and,
if the row records a bundle-inner CLI path, that path exists inside the bundle
— the check passes when the path exists or is executable, so existence alone
satisfies it and no executable mode bit is required — so a bundle whose
recorded inner CLI file is missing is not mistaken for a working install
(`src/editors.rs:230-236`, module doc at `src/editors.rs:212-214`).

#### Scenario: Complete bundle counts without any CLI on PATH

- **WHEN** no probe name for `android-studio` is on PATH but
  `/Applications/Android Studio.app/Contents/MacOS/studio` exists and is
  executable
- **THEN** the `android-studio` row is detected
  (`src/editors.rs:208-238`, test at `src/editors.rs:352-359`)

#### Scenario: Empty leftover bundle does not count

- **WHEN** `<stem>.app` exists as a directory but the row's recorded inner CLI
  path is missing inside it
- **THEN** the bundle check fails for that row
  (`src/editors.rs:224-238` — neither `exists()` nor `is_executable()` holds
  for an absent path)

#### Scenario: Bundle check is false off macOS

- **WHEN** detection runs on a non-macOS platform
- **THEN** the bundle check always answers false and the CLI probe is the only
  source of installation (`src/editors.rs:240-244`)

### Requirement: Catalog order preserved

Detection SHALL return the installed rows in catalog order by filtering, never
sorting or reordering, so the picker's menu order follows the catalog.

#### Scenario: Multiple hits keep catalog relative order

- **WHEN** both a row earlier in the catalog and a row later in the catalog
  are installed
- **THEN** the detected list names them in the same relative order as the
  catalog (`src/editors.rs:167,187-195`, test
  `detect_keeps_probe_hits_in_catalog_order` at `src/editors.rs:343-350`)

### Requirement: Probe names shared with launch

The probe-name list used by detection (primary command then aliases) SHALL be
the same one the GUI launcher's `find_cli` consults, so a row detected as
installed through a given name is launchable through that name.

#### Scenario: One list, two consumers

- **WHEN** detection or `find_cli` needs the names identifying a row
- **THEN** both enumerate `editors::probe_names` — `cmd` followed by `alts` —
  rather than maintaining separate lists (`src/editors.rs:176-182`,
  `src/launch.rs:14,50-55`)

### Requirement: Detected list drives the picker

The detected list SHALL be handed to the picker as its display list, one row
per detected editor, in catalog order; whether and when the dialog opens is
owned by the editor-picker capability.

#### Scenario: List handed over unchanged

- **WHEN** detection returns N rows in catalog order in the picker flow
- **THEN** `run_picker` passes exactly those N rows, in the same order, as the
  picker's display list (`src/main.rs:110,118`)

### Requirement: Zero detected editors fails before the dialog

When no catalog row is installed, the picker flow SHALL return exactly one
error before opening any dialog; the error names every catalog editor's
display name and tells the user where an editor would be found (its CLI on
PATH or, on macOS, its app bundle in `/Applications` or `~/Applications`); the
process exits with status 1 printing that error under the `open-in: ` prefix.

#### Scenario: Empty machine

- **WHEN** detection returns an empty list in the picker flow
- **THEN** `run_picker` returns an error naming every catalog display name and
  the install-location hint, of the form `no supported editor was found;
install one of: …`, immediately; no dialog opens, and `main` prints it to
  stderr prefixed with `open-in: ` and exits 1 (`src/main.rs:110-113,49-53,
197-205`, test at `src/main.rs:286-296`)
