# AGENTS.md

Plugin for [herdr](https://herdr.dev) that opens the current workspace in a picked editor. Behaviour is documented in [README.md](README.md) and [docs/how-it-works.md](docs/how-it-works.md); the editor catalog lives in `src/editors.rs`. Rust; sources under `src/`; single runtime dependency: `serde_json`.

## Commands

Checks: see [docs/development.md](docs/development.md) → Checks.

Dev loop: `herdr plugin link .`, then `herdr plugin action invoke open-in.open`. `herdr plugin log list --plugin open-in` lists the plugin's action records (the `open-in.open` invocation's exit status, stdout and stderr), not the popup pane's stderr. Rebuild before invoking; `plugin link` does not build.

## Gotchas

- The version appears in `Cargo.toml` and `herdr-plugin.toml`; keep both in sync (release tags verify both).
- Build the release binary before `herdr plugin link` testing; a linked plugin does not rebuild itself.
- The action id is `open-in.open`; it opens the picker as a `popup` pane through the `picker` entrypoint, and a terminal editor runs the `editor` entrypoint in a new `tab`. The picker is reachable only through the popup: running the binary directly exits 1 with `neither HERDR_PLUGIN_ENTRYPOINT_ID nor HERDR_PLUGIN_ACTION_ID is set`.
- Picker code is `#[cfg(unix)]` (`src/picker.rs`), raw mode goes through an `stty` child, and a non-unix build compiles a stub that returns an error, while CI still runs the test matrix on `windows-latest`.
- A GUI row is launched from its CLI when one is found; failing that, its macOS app bundle is opened with `open -n -a` (Android Studio ships no `studio` CLI, so it needs the bundle). See `src/launch.rs`.
- If the plugin ever needs to call back into herdr, use `HERDR_BIN_PATH`, not a bare `herdr`.
- `min_herdr_version` is checked against the running herdr binary; an older local herdr refuses to link.
