# editor-picker — delta for simplify-editor-picker

## REMOVED Requirements

### Requirement: Frame rendering

### Requirement: Key handling

### Requirement: Raw mode through stty with guaranteed restore

### Requirement: Preselect and always-open dialog

### Requirement: Picker outcome contract

### Requirement: Non-unix stub

### Requirement: Theme palette resolution

### Requirement: Last-used check mark

### Requirement: Help footer

### Requirement: Terminal size and scroll window

## ADDED Requirements

### Requirement: Compact picker dialog

The picker SHALL render the available editor choices as a compact, borderless ratatui list with terminal-default colors and basic selected-row styling. The popup SHALL be 38 columns wide and SHALL include one blank row above and below the list. The selected row SHALL begin with `›`. The picker SHALL omit the help footer and last-used marker. Rendering and selection SHALL be testable independently, including rendering with a ratatui test backend.

#### Scenario: Render choices in a test backend

- **WHEN** the renderer receives multiple editor names and a selected index with a ratatui test backend
- **THEN** the backend buffer contains the names in order and distinguishes the selected row without a live terminal

#### Scenario: Render without herdr theme configuration

- **WHEN** the picker renders a dialog while herdr theme configuration is missing, malformed, or names any theme
- **THEN** it does not read that configuration and uses terminal-default colors with no help footer or last-used marker

#### Scenario: Keep selection visible in long lists

- **WHEN** the editor list exceeds the available terminal area and selection moves through it
- **THEN** ratatui keeps the selected editor visible

### Requirement: Selection controls

The picker SHALL handle Up and Down by moving the selected row one position within the list bounds, Enter by selecting the highlighted editor, and Escape by cancelling. It SHALL NOT treat `j`, `k`, or `q` as controls. Selection updates SHALL be testable independently from rendering.

#### Scenario: Move down and select

- **WHEN** the user presses Down and then Enter while the first of multiple rows is selected
- **THEN** the picker returns the second editor

#### Scenario: Movement stays within list bounds

- **WHEN** the user presses Up at the first row or Down at the last row
- **THEN** the selected row remains unchanged

#### Scenario: Escape cancels without launching

- **WHEN** the user presses Escape while the dialog is open
- **THEN** the picker returns a cancelled outcome and the caller does not launch an editor

#### Scenario: Unsupported keys have no effect

- **WHEN** the user presses `j`, `k`, or `q`
- **THEN** the selection does not move and the picker does not cancel

### Requirement: Dialog routing and last-choice behavior

The caller SHALL launch the only available editor directly without opening the dialog. When multiple editors are available, it SHALL open the dialog and preselect the editor whose id matches the persisted last choice; when no id matches or no last choice exists, it SHALL select the first editor. On selection, the caller SHALL preserve the existing best-effort last-choice save and editor-launch flow.

#### Scenario: Launch a single available editor directly

- **WHEN** exactly one editor is available
- **THEN** the caller launches it without rendering or entering the picker dialog

#### Scenario: Open the dialog for multiple editors

- **WHEN** more than one editor is available
- **THEN** the caller opens the picker dialog and waits for selection or cancellation

#### Scenario: Preselect the matching last choice

- **WHEN** multiple editors are available and the persisted last-choice id matches one editor
- **THEN** that editor is selected initially

#### Scenario: Default to the first editor

- **WHEN** multiple editors are available and no persisted last-choice id matches
- **THEN** the first editor is selected initially

#### Scenario: Save and launch the selected editor

- **WHEN** the user selects an editor from the dialog
- **THEN** the caller saves the choice best-effort and launches that editor

### Requirement: Terminal lifecycle

On supported runtime platforms, the picker SHALL use ratatui with crossterm for input, rendering, terminal lifecycle, and terminal sizing. It SHALL NOT invoke `stty`, parse terminal escape sequences in project code, or implement a project-owned frame-sizing or scroll-window system. The terminal SHALL be restored before the picker returns after selection, cancellation, or input/rendering failure.

#### Scenario: Restore the terminal on exit

- **WHEN** the dialog returns after selection, cancellation, or an input/rendering error
- **THEN** terminal state is restored before the caller continues

### Requirement: CI target coverage

The picker implementation and its tests SHALL compile and run on every current CI target, including Windows.

#### Scenario: Build and test on every CI target

- **WHEN** CI builds and tests the picker on any current target, including Windows
- **THEN** the ratatui/crossterm implementation and its platform-independent tests compile and pass

### Requirement: Runtime plugin platforms

Runtime plugin support SHALL remain limited to Linux and macOS, as declared in `herdr-plugin.toml`.

#### Scenario: Preserve runtime plugin platforms

- **WHEN** the plugin manifest declares supported runtime platforms
- **THEN** it continues to list Linux and macOS only

### Requirement: Picker outcomes

The picker SHALL return the selected editor on Enter, a cancelled outcome on Escape, and an error when terminal setup, input, or rendering fails. An empty editor list SHALL return an error before rendering. The caller SHALL exit successfully without output on cancellation and SHALL report picker errors with a nonzero exit status.

#### Scenario: Reject an empty list before rendering

- **WHEN** the picker receives an empty editor list
- **THEN** it returns an error and writes no dialog frame

#### Scenario: Report terminal errors to the caller

- **WHEN** terminal setup, input, or rendering fails
- **THEN** the caller reports the picker error and exits with a nonzero status
