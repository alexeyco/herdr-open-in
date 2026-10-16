# Restyle the picker to the herdr theme dialog

## Why now

The popup picker currently renders a bare list: cyan `▸` + bold name, byte-length
padding, no background highlight, no footer, no scrolling
(`src/picker.rs:164-196`, spec `editor-picker` "Frame rendering"). The result
looks nothing like herdr's own settings dialogs — in particular the theme
chooser (`settings_overlay.rs` in herdr 0.9.5) that users see right next to it.
This change restyles the picker so that, side by side with herdr's UI, it reads
as part of the same application: one column, `▸` marker, full-row accent
background on selection, `✓` on the last-used editor, a muted help line, and a
minimal bottom-scroll window sized to the real terminal.

Key and selection behavior, editor detection, launching, and persisted state
are untouched.

## What changes

- **Theme-aware colors.** Resolve `[theme] name` from herdr's `config.toml`
  (`$HERDR_CONFIG_PATH` or `~/.config/herdr/config.toml`) with the `toml` crate
  — the consciously accepted second runtime dependency — and map it to one of
  the 18 built-in herdr palettes (truecolor, embedded from herdr 0.9.5).
  `terminal`, a missing file, a parse error, or an unknown name fall back to an
  ANSI-16 palette (accent→blue, overlay1→dark gray, text/panel_bg→terminal
  defaults).
- **Frame restyle.** Rows become `" ▸ name ✓"`-style: selected row is bold with
  fg=panel_bg on bg=accent spanning the full terminal width; unselected rows are
  fg=text on bg=panel_bg padded to full width; the last-used editor row carries
  a ` ✓` suffix; a help footer ` ↑↓ select  enter open  esc cancel` in
  fg=overlay1 sits at the bottom. No border, no header — herdr draws the popup
  frame and title.
- **Scrolling.** Read `stty size` before every frame; draw only the window that
  fits, with `scroll = selected.saturating_sub(visible - 1)` (herdr's own
  formula). No scrollbars or position indicators.
- **Bottom-pinned footer.** The frame now always fills the whole drawable area:
  blank top line, scroll window, blank filler, one mirror blank spacer, then the
  footer as the last line; the blank lines carry the `panel_bg` background (no
  foreground) so the popup reads as one continuous panel. The visible row
  budget becomes `rows - 3`; the frame omits a trailing newline so the
  full-height output does not scroll one row up.
- **Alphabetical catalog.** `src/editors.rs` catalog entries are reordered by
  display name, case-insensitively (Android Studio … Zed); menu order still
  follows catalog order, so detection and picker contracts need no delta.
- **Manifest.** Raise the picker popup height from `14` to `16`
  (`herdr-plugin.toml:17`) so the full 11-entry catalog + blank top line +
  spacer + help footer fit without scrolling — at exact capacity.
- **Tests and docs.** ANSI-snapshot unit tests pin the exact SGR frames; the old
  cyan-arrow frame tests are rewritten; README and docs describe the new look.
- **Non-goals.** Mouse support, live theme preview, `auto_switch`, and
  `[theme.custom]` overrides remain explicitly out of scope.

## Impact on capabilities

| Capability                                                                                                                  | Impact                                                                                                                                                                                                                        |
| --------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `editor-picker`                                                                                                             | MODIFIED "Frame rendering"; ADDED "Theme palette resolution", "Last-used check mark", "Help footer", "Terminal size and scroll window". Key handling, stty raw-mode/restore, outcome contract, preselect mechanics unchanged. |
| `plugin-registration`                                                                                                       | MODIFIED "Manifest declares picker and editor panes": popup height 14 → 16.                                                                                                                                                   |
| `editor-catalog`                                                                                                            | ADDED "Catalog entries are ordered alphabetically": source list reordered by display name; ids and all per-row fields unchanged.                                                                                              |
| `editor-detection`, `last-choice-state`, `workspace-context`, `herdr-pane-control`, `editor-launch`, `terminal-editor-exec` | Unchanged. The `✓` marker reuses the same last-choice data; launch behavior does not move, and detection still returns rows in catalog order — which is now alphabetical.                                                     |

New source: `src/theme.rs` (palette table + herdr config reader). `toml` becomes
a second runtime dependency in `Cargo.toml` — the "single runtime dependency:
serde_json" line in AGENTS.md/README is updated accordingly.

## Assumptions and accepted risks

ASSUMPTION: `✓` marks the row whose id equals the caller's preselect (last-used editor) id, mirroring the "current theme" mark in herdr's dialog; it does not move with the cursor; preselect mechanics are unchanged.
ASSUMPTION: Help footer wording is in the spirit of herdr (`↑↓ select` + `enter open` + `esc cancel`), muted fg=overlay1 on panel_bg; the exact wording is up to the implementation as long as the three affordances are conveyed.
ASSUMPTION: herdr draws the popup border and the "Open in" title; the plugin draws no `┌─┐` frame and no header line.
ASSUMPTION: The manifest height is set to `16` (inner 14 = 1 blank top line + 11 catalog rows + 1 blank spacer + 1 help footer — exact capacity, no spare row; catalog growth triggers scrolling); width stays `40%`.
ASSUMPTION: The palette is resolved once at picker start; no live re-read while the dialog is open.
ASSUMPTION: With the alphabetical catalog the preselect fallback (no last choice or unknown id) lands on the first row — now Android Studio instead of Zed; ordering compares display names case-insensitively with an ASCII fold, no locale collation.
REJECTED-RISK: auto_switch (light/dark automatic theme switching) is out of scope by explicit user decision; with auto_switch on, picker colors may not match the currently displayed theme.
REJECTED-RISK: overrides from `[theme.custom]` in config.toml are ignored — only built-in theme palettes are reproduced, by name.
REJECTED-RISK: palette drift when herdr updates is accepted; sync is done by manual updates of the embedded constants.
