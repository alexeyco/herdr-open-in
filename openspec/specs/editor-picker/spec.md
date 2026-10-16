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
or an error line. Its only static text is the bottom-pinned help footer;
there is no header, title, or search/filter UI.

## Requirements

### Requirement: Frame rendering

Each picker frame SHALL be written to stdout as a single string: hide-cursor
(`\x1b[?25l`), home (`\x1b[H`), clear-tail (`\x1b[J`), then exactly `height`
lines that always fill the whole drawable area: a blank top line, the scroll
window's rows (see "Terminal size and scroll window"), blank filler lines, one
blank spacer line, and the help footer (see "Help footer") as the frame's last
line. The frame SHALL NOT include a header, title, border, or any search/filter
input — the popup border and the "Open in" title are drawn by herdr, not by the
plugin. The frame SHALL end immediately after the footer line, without a
trailing line break — on a full-height frame the extra newline would scroll the
top line off the drawable area.

Each row SHALL have the form `" <marker> <name>"` with `<marker>` the `▸`
character on the selected row and a single space on all other rows, followed on
the last-used row by the ` ✓` suffix (see "Last-used check mark"). The selected
row SHALL be styled bold with foreground = the palette's `panel_bg` token and
background = the palette's `accent` token, painted with explicit SGR sequences
(`38;2;r;g;b` / `48;2;r;g;b` for truecolor palettes; ANSI-16 or no sequence per
the fallback palette rules) and SHALL be padded with spaces to the full drawable
width so the accent background spans the whole row. Unselected rows SHALL use
foreground = the palette's `text` token on background = the palette's
`panel_bg` token, padded with spaces to the full drawable width. Exactly one row
per frame SHALL carry the `▸` marker, rows SHALL appear in the order of the list
handed to the picker (restricted to the scroll window), and styling SHALL NOT
use reverse video (SGR `7`) — the previous cyan-arrow / bold-only, byte-padded,
unpainted rendering (src/picker.rs:52-71, 164-196) is replaced.

The blank lines (top, filler, spacer) SHALL be styled with background = the
palette's `panel_bg` token and no foreground, padded with spaces to the full
drawable width; under the ANSI-16 fallback (`panel_bg` = Default) they SHALL
emit no SGR at all.
(tests in `src/picker.rs`, ANSI snapshots)

#### Scenario: Multi-row frame

- **WHEN** the picker draws a frame for a list of N editors with row `i`
  selected, under a resolved truecolor palette, in a drawable area of `height`
  lines
- **THEN** stdout receives the clear sequences, the blank top line, N padded
  rows in list order of which only row `i` carries `▸` and the accent
  background with bold panel_bg-colored text, every row's background spans the
  full drawable width, then filler and one spacer blank line carry the rows
  down to the bottom of the drawable area, and the frame ends with the help
  footer as the terminal's last row

#### Scenario: Names padded to the widest

- **WHEN** the list contains names of differing lengths and the drawable width
  exceeds the longest row's text
- **THEN** every row is followed by enough spaces to reach the full drawable
  width — which is at least the width of the longest row — all inside that
  row's background styling, and the blank lines are padded to the same width,
  so no unpainted gap remains anywhere in the frame down to the footer

#### Scenario: No reverse video

- **WHEN** any frame is rendered, selected row included
- **THEN** the emitted bytes contain no `\x1b[7m` sequence; selection contrast
  comes only from explicit foreground/background SGR pairs

#### Scenario: Footer pinned to the bottom

- **WHEN** the frame draws `drawn` rows in a drawable area of `height` lines
  with `drawn` below the visible row budget
- **THEN** the frame contains `height - 3 - drawn` blank filler lines between
  the last drawn row and one blank spacer line, the help footer is the frame's
  final line at the bottom of the drawable area, the frame is exactly `height`
  lines in total, and the frame's final byte is the last character of the
  footer line, with no trailing `\r\n`

#### Scenario: Blanks painted as panel

- **WHEN** a truecolor palette is in effect and any frame is drawn
- **THEN** each blank line (top, filler, spacer) carries only the `panel_bg`
  background SGR plus padding spaces — no foreground sequence — so the popup
  reads as one continuous panel instead of striped unpainted gaps
- **AND** under the ANSI-16 fallback those lines emit no SGR at all

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

### Requirement: Theme palette resolution

Before drawing its first frame the picker SHALL resolve a palette with tokens
`accent`, `panel_bg`, `text`, `overlay1` by reading `[theme] name` from the
herdr config — `$HERDR_CONFIG_PATH` when set, else
`~/.config/herdr/config.toml` — parsing it with the `toml` crate. A name
matching one of the 18 herdr built-in themes SHALL use that theme's embedded
truecolor palette, transcribed from herdr 0.9.5 with the version cited in
code; `terminal`, a missing or unreadable config, invalid TOML, or an unknown
name SHALL select the ANSI-16 fallback palette. The fallback check takes
precedence over the built-in table: the name `terminal` selects the ANSI-16
fallback even if it turns out to be among the 18 built-in names (task 1.3
verifies the list). The palette SHALL be resolved
once and SHALL NOT be re-read while the dialog is open; `[theme.custom]`
overrides are out of scope and ignored.

#### Scenario: Built-in theme honored

- **WHEN** the herdr config contains `[theme]` with `name = "catppuccin"`
- **THEN** the picker renders frames using the embedded catppuccin truecolor
  palette (accent `#89b4fa`, panel_bg `#181825`, text `#cdd6f4`)

#### Scenario: Fallback palette rendering

- **WHEN** the ANSI-16 fallback palette is in effect
- **THEN** `accent` renders as ANSI blue, `overlay1` as ANSI dark gray (bright
  black), and `text` and `panel_bg` emit no foreground or background SGR at
  all, leaving the terminal's own colors

#### Scenario: HERDR_CONFIG_PATH override

- **WHEN** `HERDR_CONFIG_PATH` points at a config file with `name = "nord"`
- **THEN** the picker resolves the nord palette from that file, ignoring
  `~/.config/herdr/config.toml`

#### Scenario: Missing or broken config falls back

- **WHEN** the config file does not exist, cannot be read, or is not valid TOML
- **THEN** the picker uses the ANSI-16 fallback palette and still opens normally

#### Scenario: Terminal or unknown name falls back

- **WHEN** the resolved theme name is `terminal` or any name not among the 18
  built-ins
- **THEN** the picker uses the ANSI-16 fallback palette: selected rows get an
  ANSI blue background with no explicit foreground; unselected rows — whose
  `text` and `panel_bg` tokens carry no SGR — emit no foreground or background
  sequence at all, leaving the terminal's defaults; the footer still carries
  the `overlay1` bright-black foreground, with no background SGR

### Requirement: Last-used check mark

The row whose editor id equals the preselect id handed by the caller (the
persisted last choice, same data as the existing pre-highlight) SHALL carry the
suffix ` ✓`, regardless of which row is selected; when no row's id matches, no
row SHALL carry the mark. The mark SHALL NOT alter pre-highlight, key handling,
or the picker outcome contract.

#### Scenario: Check mark on the last-used row

- **WHEN** the caller passes a preselect id present in the list and the user
  moves the cursor away from that row
- **THEN** the preselect row keeps its ` ✓` suffix while the `▸` marker moves

#### Scenario: No check mark without a match

- **WHEN** the caller passes no preselect, or a preselect id absent from the
  list
- **THEN** no row carries ` ✓`

### Requirement: Help footer

Every frame SHALL end with exactly one help line: `↑↓ select`, `enter open`,
and `esc cancel` separated by spaces (leading space; wording may vary as long
as the three affordances are named), fg = `overlay1` on bg = `panel_bg`, padded
to the full drawable width. It SHALL be the frame's final line, pinned to the
bottom of the drawable area directly under exactly one blank spacer line. No
other footer elements — no buttons, no key hints elsewhere.

#### Scenario: Footer present and styled

- **WHEN** any frame is drawn
- **THEN** the last line of the frame names the three affordances, is styled
  overlay1-on-panel_bg, spans the full drawable width, and sits on the
  drawable area's bottom line with one blank spacer line above it

#### Scenario: Footer is static

- **WHEN** the user navigates between rows
- **THEN** the footer text and style are identical in every frame

### Requirement: Terminal size and scroll window

Before rendering each frame the picker SHALL determine its drawable size by
running the external `stty size` program as a child process — the same
invocation pattern as the existing raw-mode `stty` usage — and parsing its
`<rows> <cols>` output; the reported size is the drawable area, and the drawable
height is the frame's `height`. If the command fails or the output cannot be
parsed, the picker SHALL assume a default 80×24 drawable area. The visible row
budget SHALL be `rows - 3` — the blank top line, the blank spacer line above
the footer, and the footer — at least 1, and the picker SHALL draw only rows
`[scroll, scroll + visible)` where `scroll = selected.saturating_sub(visible -
1)`. The frame SHALL fill the drawable area exactly: the number of blank filler
lines between the last drawn row and the spacer is recomputed each frame from
the freshly-read size. The frame SHALL NOT include scrollbars, percentage
indicators, or row numbers. Because the size is read every frame, resizing the
terminal takes effect from the next redraw.

#### Scenario: List fits — static frame

- **WHEN** the list has N rows and N is at most the visible row budget
  (`rows - 3`)
- **THEN** all N rows are drawn and no row is hidden, with blank filler and
  the spacer line padding the frame down to the footer on the last line

#### Scenario: Minimal bottom-scroll on downward selection

- **WHEN** the visible budget is `V` rows and the user moves the selection down
  to row index `V` (0-based)
- **THEN** the next frame draws rows `1..=V` only — the window scrolled by
  exactly one — with the selected row at the bottom of the window

#### Scenario: Scrolling back up

- **WHEN** the window is scrolled and the user moves the selection to the first
  row
- **THEN** the next frame draws rows `0..V` with `scroll = 0`

#### Scenario: Size read failure uses defaults

- **WHEN** `stty size` cannot be executed or returns unparsable output
- **THEN** the picker renders with an assumed 80×24 area and does not error
