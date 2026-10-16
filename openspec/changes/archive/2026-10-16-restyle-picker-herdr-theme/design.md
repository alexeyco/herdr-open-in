# Design — restyle-picker-herdr-theme

## Approach

Restyle `src/picker.rs` to reproduce, inside the plugin's PTY, the rendering of
herdr's own theme-selection dialog (`settings_overlay.rs` in herdr 0.9.5), with
colors resolved from the user's herdr theme. All logic stays in the plugin:
herdr gives plugins no theme API, the popup is a plain terminal pane, and the
plugin emits ANSI itself.

## New module: `src/theme.rs`

- `Palette { accent, panel_bg, text, overlay1 }` where each token is a
  `TokenColor` enum: `Rgb(r, g, b)` (emits `38;2;r;g;b` / `48;2;r;g;b`),
  `Ansi(code)` (emits the ANSI-16 SGR, e.g. `34`/`44` for blue, `90` for bright
  black), or `Default` (emits nothing — the terminal's own colors show through).
  `TokenColor` gets `fg_sgr()` / `bg_sgr()` helpers returning the escape prefix
  (possibly empty) so `picker.rs` never branches on theme kind.
- `BUILTIN_PALETTES: &[(&str, Palette); 18]` — the 18 built-in herdr theme
  names with the four tokens each, transcribed as truecolor hex from herdr
  0.9.5's palette definitions. The exact source file is UNVERIFIED
  (`src/app/state.rs` was guessed): task 1.3 must first locate the real
  palette table in the herdr sources and verify every theme name and hex
  against the actual file before transcribing. Each embedded palette carries
  a `// sourced from herdr 0.9.5` comment and a `THEME_SOURCE_VERSION`
  constant. Only the tokens the picker paints are
  embedded, not full palettes.
- `resolve_palette() -> Palette`: reads `$HERDR_CONFIG_PATH`, else
  `~/.config/herdr/config.toml`; parses with `toml::Value` and indexes
  `theme` → `name` (no serde structs needed for one string). Any failure —
  missing file, IO error, invalid TOML, non-string or unknown name — returns
  the ANSI-16 fallback (`accent=Ansi(blue)`, `overlay1=Ansi(bright black)`,
  `text=Default`, `panel_bg=Default`). `name == "terminal"` explicitly selects
  the fallback. Called once at picker start.

Trade-off accepted (herdr parity): herdr paints the selected row's foreground
with `contrast(palette)` = panel_bg as fg, or `surface_dim` when panel_bg is
Reset. The ANSI-16 fallback has no surface_dim; there we emit no fg sequence at
all on the selected row (bold + ANSI-blue bg still contrast). Exact parity is
only claimed for truecolor palettes.

## Frame rework (`src/picker.rs`)

Refactor `frame()` into a pure function that takes everything it renders —
`(list, selected, last_used_id: Option<&str>, palette: &Palette,
size: (rows, cols), scroll, visible)` — and returns the full frame string, so
tests can pin exact SGR bytes without a tty. One `stty size` tuple (rows and
cols together, matching `DEFAULT_SIZE`/`terminal_size`) instead of separate
`width`/`height` params keeps the pair from drifting apart. `run()` gathers
those inputs:

- Top: blank line (kept from today, gives one row of breathing space under
  herdr's border).
- Row: ` ▸ Name ✓` / `   Name` — marker per "Frame rendering", ` ✓` suffix on
  the row whose id equals the caller's preselect (that value already reaches
  the picker; no new data flow, pre-highlight mechanics unchanged).
- Selected row: `\x1b[1m` + fg `panel_bg` + bg `accent`, text then spaces
  padded to the full drawable width inside the styling; unselected: fg `text`
  bg `panel_bg`, same padding. Replace the byte-based `str::len()` padding with
  padding against `unicode-width`-free character count of the row text (the
  catalog names are ASCII; `▸`/`✓` are counted as one column each — compute
  padding from `chars().count()`, which is sufficient for this fixed catalog).
- Below the window rows: blank filler lines (`height - 3 - drawn`), then one
  blank spacer line, then the footer — the frame is always exactly `height`
  lines. All blank lines (top, filler, spacer) are painted bg `panel_bg` only,
  no fg, padded to width.
- Footer: one line ` ↑↓ select  enter open  esc cancel`, fg `overlay1`
  bg `panel_bg`, padded to width; bottom-pinned as the frame's last line, not
  newline-terminated (the frame ends on the footer's last byte).
  No header, no border (herdr draws the popup frame and title).
- Never emit `\x1b[7m`; keep a test asserting it.

Keys, raw mode, stty save/restore, outcome contract, non-unix stub: untouched.

## Terminal size and scroll

- `terminal_size()` helper: run `stty size` as a child bound the same way as
  the existing stty invocations, parse `"<rows> <cols>"`; on any error return
  the 80×24 default. Called before each frame (cheap child process; the dialog
  is not perf-sensitive).
- `visible = drawable_rows.saturating_sub(3).max(1)` (blank top line + blank
  spacer + footer); `scroll = selected.saturating_sub(visible - 1)`; draw rows
  `scroll..min(len, scroll+visible)`. Identical to herdr's minimal bottom-scroll;
  no indicators.
- The frame is padded to exactly `drawable_rows` lines: blank filler between the
  last drawn row and the spacer, recomputed per frame from the freshly-read size.

## Bottom-pinned footer

The footer sits on the drawable area's last line, directly under one blank
spacer line mirroring the blank top line. Decision: paint all blank lines
(top, filler, spacer) with bg `panel_bg` and no foreground. Rationale:
unpainted blanks would show the terminal's own background as horizontal
stripes between the panel-colored rows; bg-only blanks make the popup read as
one continuous panel under truecolor palettes. Under the ANSI-16 fallback
`panel_bg` is `Default`, so blank lines emit no SGR at all — the terminal's
natural look is preserved there.

The frame emits `height - 1` line breaks, not one per line: a full-height frame
followed by a trailing newline scrolls the terminal exactly one row up, pushing
the top blank padding off the drawable area and leaving a spurious blank line
below the footer; ending the frame on the footer's last character keeps the top
padding visible and docks the footer to the last row.

Capacity consequence (accepted, by design): the visible budget is now
`rows - 3`, so the popup's 14 inner lines hold exactly 11 catalog rows —
no spare row. The current catalog still fits without scrolling, but any
catalog growth triggers the scroll window. Height stays 16: the user asked
for layout, not resize, and a taller popup would add only filler lines.

## Catalog order

The catalog list in `src/editors.rs` is reordered alphabetically by display
name, compared case-insensitively (ASCII fold). Chosen over sorting at
render time so the existing contracts — detection filters without reordering
("Catalog order preserved") and the picker drawing rows in list order — stay
true verbatim; the delta is confined to the source list. Case-insensitive
because that is the alphabetical order a human expects: `helix` after Cursor,
`nvim` after IntelliJ IDEA, and `VS Code` between `vim` and `WebStorm`. Ids,
kinds, argv, and launch logic are untouched; display names keep their current
spelling (`nvim`, `vim`, `helix` render lowercase). Consequence: the preselect
fallback (no last choice / unknown id) highlights the first row, which is now
Android Studio instead of Zed — behavior that follows from the order; the
last-choice-state spec itself is unchanged.

## Manifest

`herdr-plugin.toml:17` height `14` → `16`: inner 14 lines = 1 blank top +
11 catalog rows + 1 blank spacer + 1 footer — exact fit, no spare row. Width
`40%` unchanged. The plugin-registration baseline spec pins 40%×14 as a
contract; the MODIFIED delta in this change updates it, and the manifest
change ships in the same wave as the spec sync.

## Tests

Existing frame tests pin the old cyan/bold-only format and are rewritten to the
new renderer. New ANSI-snapshot tests (pure `frame()` calls, injected palette):

- exact SGR bytes for a selected row under a fixture truecolor palette
  (catppuccin: bg `48;2;137;180;250`, fg `38;2;24;24;37`, bold), an unselected
  row, and the footer;
- full-width padding: every styled row ends at exactly `width` columns;
- ` ✓` on the last-used row that stays put when selection moves; no mark
  when preselect is unknown;
- scroll window frames: selection at index `V` renders rows `1..=V`; back at 0
  renders `0..V`; fits-list frame lists all rows;
- palette resolver: built-in name, `HERDR_CONFIG_PATH` override, missing file,
  invalid TOML, unknown name, `terminal` → fallback (tests write temp TOML
  fixtures);
- retain the no-reverse-video assertion (`\x1b[7m` absent) and the exactly-one
  `▸` assertion.

`stty size` and the config reader are factored behind small functions so frame
tests never need a tty; the size-failure default path is covered by feeding
`terminal_size` unparsable input in a unit test where possible, else marked
UNVERIFIED until the eyeball check.

## Docs

`README.md:42-56` and `docs/how-it-works.md:109-121` describe the old look and
must show the new frame (themed rows, ✓, footer, scroll, theme source and its
limits: no `[theme.custom]`, no auto_switch, manual palette sync). The
"single runtime dependency: serde_json" claims in `README.md`/`AGENTS.md` gain
`toml`.

## Success criteria mapping

Unit tests pin the ANSI frames (exact SGR); final acceptance is an eyeball
comparison against herdr's settings theme dialog on the same machine and theme:
same accent highlight, same muted footer tone, same ✓ idiom, one-column list.

## Key decisions and accepted risks

REJECTED-RISK: auto_switch (light/dark automatic theme switching) is out of scope by explicit user decision; with auto_switch on, picker colors may not match the currently displayed theme.
REJECTED-RISK: overrides from `[theme.custom]` in config.toml are ignored — only built-in theme palettes are reproduced, by name.
REJECTED-RISK: palette drift when herdr updates is accepted; sync is done by manual updates of the embedded constants (mitigated by the version citation in `src/theme.rs`).
ruling: degenerate-height clamp in `frame()` is one-sided (filler and spacer drop first; a hand-fed `height < drawn + 3` can emit more lines than `height`) — left as is by user decision: unreachable through `run_with` (the `visible_rows` budget keeps `drawn` inside the area), no spec scenario covers sub-chrome heights, and the doc comment states the degradation.
ruling: stderr on closing error paths prints after the unterminated footer line (the first byte wraps one row) — left as is by user decision: cosmetic on a pane that exits immediately, diagnostics in the action log are unaffected, and emitting a stray newline would break the "exactly `height` lines, footer last, no trailing break" frame contract.
ASSUMPTION: `✓` marks the row whose id equals the caller's preselect (last-used) id and does not move with the cursor.
ASSUMPTION: footer wording is implementation's choice within the three affordances.
ASSUMPTION: herdr draws the popup border/title; the plugin draws neither.
ASSUMPTION: manifest height 16 (inner 14 = 1 blank top + 11 rows + 1 spacer + 1 footer — exact capacity, no spare row).
ASSUMPTION: palette resolved once at picker start, no live re-read.
