# editor-catalog

## Purpose

The editor catalog is the compile-time fixed set of editors the open-in plugin
can offer. It lives in `src/editors.rs` as a single constant list of rows, and
nothing at runtime can add or remove a row: the plugin reads no configuration
that changes the catalog's membership. Each row is a small contract carrying a
stable id, a display name, a kind (GUI or Terminal), a primary CLI command, an
optional list of alias command names, an argv template with exactly one `<dir>`
placeholder, and, for GUI rows, a macOS app-bundle stem plus the optional path
of a CLI inside that bundle. The stable ids are the externally visible surface:
they are what the picker and the tab handoff pass through `OPEN_IN_EDITOR`,
what the last-editor state file persists, and what error messages list. Which
rows exist and in which order the menu renders them — alphabetically by
display name — is a contract pinned below; the per-id id/kind/argv shape and
the structural invariants below are the contract that detection, launching,
and state handling depend on.

## Requirements

### Requirement: Catalog is compile-time fixed

The catalog SHALL be a constant list embedded at build time; the plugin SHALL
not add or remove catalog rows based on any runtime configuration, state file,
or environment variable.

#### Scenario: No runtime configuration alters membership

- **WHEN** the plugin runs detection or shows the picker on any machine
- **THEN** the candidate set of editors is exactly the rows of the built-in
  catalog, and no user setting can introduce an editor that is not a catalog
  row (`src/editors.rs:53-165`)

### Requirement: Row shape

Every catalog row SHALL carry a stable id, a non-empty display name, a kind of
either `Gui` or `Terminal`, a non-empty primary CLI command name, an ordered
(possibly empty) list of alias command names, a non-empty argv template, and an
optional macOS bundle stem with an optional bundle-inner CLI path.

#### Scenario: Structural fields are populated

- **WHEN** any catalog row is inspected
- **THEN** its `id`, `name`, `cmd`, and `args` are non-empty, and its kind is
  one of `Gui` or `Terminal` (`src/editors.rs:23-51`, test
  `every_row_has_a_non_empty_name_command_and_argv` at `src/editors.rs:288-295`)

#### Scenario: Bundle fields follow kind

- **WHEN** a row's kind is `Gui`
- **THEN** the row declares a bundle stem, and when a row's kind is `Terminal`
  the row declares no bundle (`src/editors.rs:44-46`, tests at
  `src/editors.rs:305-310` and `src/editors.rs:369-380`)

#### Scenario: Bundle-inner CLI is relative and paired

- **WHEN** a row records a bundle-inner CLI path
- **THEN** that path is non-empty, relative to the bundle root, and the row
  also records a bundle stem (`src/editors.rs:47-50`, test at
  `src/editors.rs:312-321`)

### Requirement: Single directory placeholder

Each row's argv template SHALL contain the `<dir>` placeholder exactly once;
launch code substitutes the resolved workspace directory at that element when
the editor is started (`src/main.rs:183-195`, `src/launch.rs:58-70`).

#### Scenario: Placeholder count per row

- **WHEN** any catalog row's argv template is inspected
- **THEN** exactly one element equals the `<dir>` placeholder
  (`src/editors.rs:10-12`, test at `src/editors.rs:297-303`)

### Requirement: Per-id kind and argv shape

The stable id of each editor SHALL be bound to a fixed kind and argv shape,
because the launch paths depend on them: the VS Code-family rows (`zed`,
`code`, `cursor`, `windsurf`) open a directory with the `-n <dir>` argv
template, while the JetBrains-family rows (`idea`, `webstorm`, `pycharm`,
`android-studio`) and the terminal rows (`nvim`, `vim`, `helix`) open it with a
bare `<dir>` template. The `helix` row's primary command is `hx`, and the
`android-studio` row's primary command is `studio`.

#### Scenario: VS Code-family argv

- **WHEN** the plugin builds argv for row `zed`, `code`, `cursor`, or
  `windsurf` with a resolved directory
- **THEN** the result is `-n` followed by the directory
  (`src/editors.rs:61,71,81,91`)

#### Scenario: JetBrains-family and terminal argv

- **WHEN** the plugin builds argv for row `idea`, `webstorm`, `pycharm`,
  `android-studio`, `nvim`, `vim`, or `helix` with a resolved directory
- **THEN** the result is the bare directory (templates at
  `src/editors.rs:101,111,121,131,141,151,161`,
  substitution logic at `src/main.rs:183-195`), and the rows exec their
  recorded primary commands — `hx` for `helix` (`src/editors.rs:159`) and
  `studio` for `android-studio` (`src/editors.rs:129`)

#### Scenario: Kind assignment is stable

- **WHEN** any row is consulted for its kind
- **THEN** `nvim`, `vim`, and `helix` are `Terminal` and every other catalog id
  is `Gui` (`src/editors.rs:135-164`, test at `src/editors.rs:369-380`)

### Requirement: Lookup by stable id

Resolving a catalog row by id SHALL match the id exactly against the catalog,
and an unknown id SHALL be rejected with an error that names the offending id
and lists every known catalog id.

#### Scenario: Known id resolves

- **WHEN** a catalog id such as `nvim` is looked up
- **THEN** the corresponding row is returned (`src/main.rs:169-171`, test at
  `src/main.rs:247-255`)

#### Scenario: Unknown id is rejected

- **WHEN** an id that is not a catalog id is looked up
- **THEN** the lookup fails with `unknown editor \`<id>\`; known editors: <all
  ids>` listing every catalog id (`src/main.rs:159-165,173-180`, test at
`src/main.rs:262-275`)

### Requirement: Alias command names participate in the contract

A row MAY declare alias CLI names probed after its primary command, and the
alias list SHALL be shared so that detection and GUI launch agree on which
names identify a row. Today only `zed` declares an alias (`zeditor`, for
Debian/Ubuntu packaging); every other row's probe-name list is just its
primary command.

#### Scenario: Alias appended after primary command

- **WHEN** the probe-name list for row `zed` is enumerated
- **THEN** it yields `zed` first, then `zeditor`, and for row `vim` it yields
  only `vim` (`src/editors.rs:35-40,60,176-182`, test at
  `src/editors.rs:323-341`)

#### Scenario: Stable ids are the external surface

- **WHEN** the plugin persists the last choice or hands a choice to the
  terminal-editor tab
- **THEN** it uses the catalog id string unchanged, so ids form the contract
  surface for the state file and the `OPEN_IN_EDITOR` environment variable
  (`src/editors.rs:26-27`, `src/main.rs:43-47,143`, `src/state.rs:1-29`)

### Requirement: Catalog entries are ordered alphabetically

The catalog SHALL list its editor entries ordered by display name, compared
case-insensitively (ASCII fold, no locale collation). Only the order of the
source list changes: ids, kinds, commands, and every other per-entry field
stay as they are. The picker's display order follows from the existing
"Catalog order preserved" contract (editor-detection) and the menu's
catalog-order rule, so no other capability needs a delta.

#### Scenario: Alphabetical display order

- **WHEN** the picker receives all catalog entries
- **THEN** the rows appear in the order Android Studio, Cursor, helix,
  IntelliJ IDEA, nvim, PyCharm, vim, VS Code, WebStorm, Windsurf, Zed —
  the catalog's display names in case-insensitive alphabetical order (VS
  Code sits between vim and WebStorm)
