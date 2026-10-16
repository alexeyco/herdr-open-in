# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.3.0

### Added

- The picker now matches herdr's theme dialog: the selected editor's row is highlighted across the full width of the popup in the theme's accent color with a `▸` cursor, the last-used editor's row carries a ` ✓` mark, and a static help footer (`↑↓ select  enter open  esc cancel`) is docked to the bottom of the popup under one blank spacer line.
- Theme colors are resolved from `[theme] name` in the herdr config file (`$HERDR_CONFIG_PATH`, else `~/.config/herdr/config.toml`): the palettes of all 18 built-in herdr 0.9.5 themes are embedded, and `terminal`, an unreadable config, or an unknown theme falls back to an ANSI-16 palette. `[theme.custom]` overrides and `auto_switch` are not followed.
- Minimal bottom-scroll when the list is taller than the popup: the real popup size is re-read via `stty size` every frame (80×24 assumed when unavailable), so resizing the terminal is handled and no scrollbars or position indicators are shown.

### Changed

- The catalog rows are now alphabetical (case-insensitive): Android Studio, Cursor, helix, IntelliJ IDEA, nvim, PyCharm, vim, VS Code, WebStorm, Windsurf, Zed. With no remembered choice the picker opens on the first row, Android Studio, instead of Zed.
- The picker popup is 16 lines tall instead of 14: the full eleven-row catalog, the blank top line, the spacer and the footer now fit exactly without scrolling.
- Added `toml` as a second runtime dependency for reading the herdr config file.
- Version bumped to `0.3.0`.

## 0.2.0

### Added

- Editor picker: the `open-in.open` action opens a popup listing the editors installed on this machine out of an eleven-row catalog (Zed, VS Code, Cursor, Windsurf, IntelliJ IDEA, WebStorm, PyCharm, Android Studio, nvim, vim, helix), detecting each one by probing its CLI on `PATH` and, on macOS, by looking for its app bundle in `/Applications` and `~/Applications`.
- Choosing a GUI editor opens the workspace directory in it as a detached process, while nvim, vim and helix open a new herdr tab in the current workspace.
- The last picked editor is saved to `$HERDR_PLUGIN_STATE_DIR/last-editor` and pre-marked the next time the picker opens; a single detected editor still shows the dialog.
- A machine with no catalog editor installed exits with status 1 after one error line naming the editors that were searched for.

### Changed

- Renamed the plugin id and binary from the earlier Zed-only name to `open-in` / `herdr-open-in`. This is breaking for existing keybindings such as `prefix+shift+o`: the qualified action id is now `open-in.open`.
- Version bumped to `0.2.0`.

## 0.1.0

### Added

- `open` action that resolves the workspace directory from the herdr plugin context (`worktree.checkout_path` → `workspace_cwd` → `focused_pane_cwd`) and opens it in a new Zed window via `zed -n <dir>`.
- Zed CLI lookup: `zed` on `PATH`, then known install locations on macOS and Linux.
- Documentation: README, how it works, development guide, contributing guide.
- CI: fmt, clippy, tests, and release build on Linux, macOS, and Windows; release workflow publishing binaries for `v*` tags.
