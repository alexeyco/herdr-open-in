# editor-picker Specification

## Purpose

The editor picker is the interactive dialog the plugin renders inside a herdr
`popup` pane: one row per detected editor, catalog order, arrow and vim
movement, Enter to select, `q` or Escape to cancel. It exists only on unix
(`src/picker.rs` is entirely `#[cfg(unix)]` with a non-unix stub), enters raw
terminal mode exclusively through an `stty` child process, and restores the
saved settings and the hardware cursor on every exit path so the terminal is
never left broken. The picker itself prints nothing about the selection; the
caller in `src/main.rs` turns the outcome into a launch, a silent clean exit,
or an error line. There is no header, help text, or search/filter UI.

## Requirements

### Requirement: Frame rendering

Each picker frame SHALL be written to stdout as a single string: hide-cursor
(`\x1b[?25l`), home (`\x1b[H`), clear-tail (`\x1b[J`), one blank line, one row
per editor, then exactly one trailing blank line. The selected row SHALL be
marked by a cyan (`\x1b[36m`) `▸` arrow followed by a space and bold
(`\x1b[1m`) name; unselected rows SHALL carry a two-space prefix; every name
SHALL be padded with spaces to the byte length of the widest name in the
list — the width is computed with `str::len()` (src/picker.rs:165-169,187),
so it counts bytes, not characters or display columns; rows
SHALL appear in the order of the list handed to the picker; exactly one row
per frame SHALL carry the arrow. The frame SHALL NOT include a header, help
text, or any search/filter input (src/picker.rs:34-71, 159-196; tests
src/picker.rs:488-507, 641-719).

#### Scenario: Multi-row frame

- **WHEN** the picker draws a frame for a list of N editors with row `i`
  selected
- **THEN** stdout receives the clear sequences, one blank line, N padded rows
  in list order with only row `i` marked by the cyan arrow and bold text, and
  one trailing blank line

#### Scenario: Names padded to the widest

- **WHEN** the list contains names of differing lengths
- **THEN** every rendered row's name is followed by enough spaces to reach the
  width of the longest name

### Requirement: Key handling

The picker SHALL decode navigation and choice keys as follows: `Up`/`k` move
the cursor one row up and `Down`/`j` one row down, both saturating at the ends
of the list; `\r` or `\n` selects the highlighted row; `q` cancels; a lone
Escape cancels. Arrow keys sent as CSI or SS3 sequences (`ESC [ A`/`ESC O A`
for up, `ESC [ B`/`ESC O B` for down) SHALL be recognized; an incomplete
CSI/SS3 sequence SHALL be ignored, not treated as a cancel; a byte that does
not belong to the sequence (for example the key after Alt+Escape) SHALL be
pushed back and decoded as a fresh key, while the lone Escape of an Alt+key
pair SHALL be ignored. Any other key SHALL be ignored and the picker SHALL
keep running (src/picker.rs:140-146, 226-284; tests src/picker.rs:509-595).
Ctrl-C has no special handling: raw mode disables `ISIG`, so the byte is
delivered as an ordinary ignored key.

#### Scenario: Arrow down then Enter

- **WHEN** the user presses Down and then Enter on a list whose first row is
  highlighted
- **THEN** the picker returns the second editor of the list

#### Scenario: Cursor saturates at the ends

- **WHEN** the user presses Up while the first row is highlighted, or Down
  while the last row is highlighted
- **THEN** the highlighted row does not change and the picker redraws and
  keeps waiting

#### Scenario: Unknown keys are ignored

- **WHEN** the user presses a key that is not navigation, select, or cancel
- **THEN** the picker keeps running with the cursor unchanged

#### Scenario: Alt+key does not cancel

- **WHEN** the user sends Escape immediately followed by a non-CSI/SS3 byte
  (an Alt+key sequence)
- **THEN** the Escape is ignored, the following byte is decoded as a normal
  key, and the picker does not cancel

### Requirement: Raw mode through stty with guaranteed restore

The picker SHALL enter raw mode only by running the external `stty` program:
first `stty -g` to save the current settings, then `stty raw -echo min 0 time
1`. If `stty` is missing or any invocation fails, the picker SHALL report an
error rather than silently continuing in cooked mode; `stty`'s stdin SHALL be
bound to `/dev/tty` when that device can be opened, otherwise inherited
(src/picker.rs:347-420). The saved settings SHALL be restored on every exit
path — explicitly before `run` returns and again through an RAII `Drop` guard
— and the restore SHALL be idempotent and also write the show-cursor sequence
(`\x1b[?25h`) to stdout (src/picker.rs:98-106, 339-392). Because raw mode
sets `min 0 time 1`, the picker SHALL decide the meaning of a 0-byte read by
whether stdin is a terminal: on a tty it is the 0.1 second read timeout
elapsed with no key and the loop SHALL keep running; on any other reader it is
end of input and SHALL cancel the picker immediately after drawing one frame
(src/picker.rs:15-21, 85-89, 118-148, 296-331; tests src/picker.rs:533-548,
565-569). A bare Escape on a real tty is therefore recognized as a cancel
because the next read times out rather than blocking.

#### Scenario: Missing stty is an error

- **WHEN** the `stty` program cannot be executed or exits unsuccessfully
- **THEN** the picker returns an error naming the failed `stty` invocation and
  does not draw a frame

#### Scenario: Timeout keeps the picker open

- **WHEN** a read times out with no key pending on a tty
- **THEN** the picker redraws and waits for the next key without cancelling

#### Scenario: Closed stdin cancels after one frame

- **WHEN** the picker reads from a stdin that is not a tty and input ends
- **THEN** the picker cancels (returns `Ok(None)`) after exactly one frame has
  been written

#### Scenario: Restore is idempotent

- **WHEN** both the explicit restore in `run` and the drop of the guard execute
- **THEN** settings are applied and the cursor is shown only once, and the
  second call succeeds without re-writing anything

#### Scenario: Restore failure overrides a successful pick

- **WHEN** the key loop completed successfully but restoring the terminal
  settings fails
- **THEN** the picker returns the restore error instead of the choice; if the
  key loop itself already errored, that outcome error is returned instead

### Requirement: Preselect and always-open dialog

The picker SHALL pre-highlight the row whose id equals the `preselect` value
handed by the caller (the persisted last choice, passed at
src/main.rs:117-118); an unknown id or no preselect SHALL fall back to the
first row (src/picker.rs:150-157; tests src/picker.rs:585-595). The dialog
SHALL be shown even when exactly one editor is detected — a single-item list
is rendered and fully interactive, never auto-selected (src/main.rs:115-116,
src/picker.rs:607-639 test).

#### Scenario: Known preselect is marked

- **WHEN** the caller passes a preselect id present in the list
- **THEN** that row is highlighted on the first frame, so pressing Enter
  without moving selects it

#### Scenario: Unknown preselect falls back to the first row

- **WHEN** the caller passes a preselect id that is not in the list
- **THEN** the first row is highlighted

#### Scenario: Single detected editor still opens the dialog

- **WHEN** exactly one editor is detected on the machine
- **THEN** the picker draws a one-row frame marking that row, and selection
  and cancellation keys behave per the key-handling requirement

### Requirement: Picker outcome contract

The picker SHALL return one of: `Ok(Some(editor))` when the user selects a row
with Enter; `Ok(None)` when the user cancels with `q`, a lone Escape, or end
of input; `Err` when the list is empty — an error naming the empty list, of
the form `no editors to pick from` — when `stty` fails, or when a terminal
read or draw fails (src/picker.rs:73-107,
127-138, 317-331). An empty list SHALL be rejected before any frame is drawn
(test src/picker.rs:597-605); in practice it is unreachable because the caller
fails with an error of the form no supported editor was found; install one
of: … when detection yields nothing, before opening the picker
(src/main.rs:110-113, 198-205). On the caller side,
`Ok(None)` SHALL exit the process silently with status 0 and print nothing;
`Ok(Some)` SHALL print nothing by itself and continue into state saving and
launching; `Err` SHALL be printed as `open-in: <error>` on stderr with exit
status 1 (src/main.rs:49-53, 118-129).

#### Scenario: Cancel exits silently

- **WHEN** the user cancels the picker with `q` or Escape
- **THEN** the plugin process writes nothing further and exits with status 0

#### Scenario: Picker error surfaces on stderr

- **WHEN** the picker returns an error (for example a failing `stty`)
- **THEN** the process prints `open-in: ` followed by the error message on
  stderr and exits with status 1

#### Scenario: Empty list is an error without drawing

- **WHEN** the picker is called with an empty editor list
- **THEN** it returns the error "no editors to pick from" and writes no frame

### Requirement: Non-unix stub

In non-unix builds the picker SHALL be a stub that takes no action and returns
an error naming the terminal requirement, of the form `the editor picker
requires a terminal and is only supported on unix`, which the caller surfaces
as an `open-in: `-prefixed line on stderr with exit 1
(src/picker.rs:109-116, src/main.rs:50-53). The plugin manifest restricts
supported platforms to `linux` and `macos` (herdr-plugin.toml:7).

#### Scenario: Stub build refuses the picker

- **WHEN** the picker entrypoint runs in a non-unix build
- **THEN** the stub itself produces no output — no UI, nothing on stdout — and
  returns the error immediately; the caller still prints its `open-in: `-prefixed
  error line on stderr and exits 1
