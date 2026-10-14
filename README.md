# open-in

![open-in](assets/banner.svg)

herdr plugin that opens the current workspace in the editor you pick.

[herdr](https://herdr.dev) is a terminal workspace manager. This plugin bridges it to the editors already installed on your machine: one keypress opens a picker listing every supported editor detected here, and the choice opens the current workspace directory — a GUI editor in its own window as a detached process, or nvim, vim and helix in a new herdr tab in the current workspace. It targets the herdr plugin API v1 and runs on Linux and macOS (Windows untested, and no Windows build is published).

## Requirements

- herdr >= 0.9.0
- Rust toolchain (the plugin builds itself on install)
- At least one supported editor installed — its CLI on `PATH` or, on macOS, its app bundle in `/Applications` or `~/Applications` (see the [editor catalog](#editor-catalog)); detection probes the CLI with `--version`
- No Python or Perl: the plugin is a single Rust binary whose only dependency is `serde_json`. The picker reads the terminal through `stty`, which every POSIX system ships.

Zed's CLI is installed from Zed's command palette: run `cli: install cli binary`. See the [Zed CLI reference](https://zed.dev/docs/reference/cli).

## Install

```
herdr plugin install alexeyco/herdr-open-in
```

## Usage

Bind a key to the action in your herdr config:

```toml
[[keys.command]]
key = "prefix+shift+o"
type = "plugin_action"
command = "open-in.open"
description = "Open workspace in an editor"
```

Any chord works; see the [herdr keyboard docs](https://herdr.dev/docs/keyboard/) for picking one that your terminal doesn't swallow. You can also invoke the action from the CLI:

```
herdr plugin action invoke open-in.open
```

### The picker

The action opens a focused popup pane listing exactly the editors detected on this machine, one row per editor, in catalog order. While a picker is open, herdr rejects a second popup with `ui_busy`.

- `↑`/`↓` (or `k`/`j`) move the selection.
- Enter launches the selected editor on the workspace directory.
- Escape or `q` cancels: the popup closes, nothing is launched and the action exits 0.

A single detected editor still shows the dialog, so `q` remains a way to back out. With no detected editor the picker exits 1 after writing exactly one error line naming every supported editor to stderr inside the popup, and the popup closes. A popup pane's stderr is not captured by herdr, so that line is not recorded anywhere in `herdr plugin log list`. That command lists the plugin's action records — the `open-in.open` invocation's own exit status, stdout and stderr — not the picker popup it opens:

```
herdr plugin log list --plugin open-in
```

To read the picker's error line directly, run the built binary from a terminal with `HERDR_PLUGIN_ENTRYPOINT_ID=picker` and a synthetic `HERDR_PLUGIN_CONTEXT_JSON`; the zero-editor check runs before the dialog starts, so it prints the line and exits 1.

### What gets opened

The directory comes from the plugin context: the worktree checkout path first (so a focused subdirectory still opens the repo root), then the workspace cwd, then the focused pane cwd. Details in [docs/how-it-works.md](docs/how-it-works.md).

- **GUI editor** — started detached on that directory and left alone; the popup returns as soon as the process is spawned. The CLI is preferred (`zed -n <dir>`, `code -n <dir>`, `cursor -n <dir>`, `windsurf -n <dir>`, `idea <dir>`, `webstorm <dir>`, `pycharm <dir>`, `studio <dir>`). When the CLI is not on `PATH`, the known install locations are probed too (`/usr/local/bin` and the app bundle's inner CLI on macOS; `/usr/bin`, `/usr/snap/bin`, `/snap/bin` on Linux), and a macOS app bundle with no usable CLI — Android Studio on a typical install — is opened with `open -n -a "<bundle>" <dir>`.
- **nvim, vim and helix** — opened in a new herdr tab in the current workspace, running in the foreground inside that tab. The tab is requested from herdr and outlives the picker.

The last editor you choose is written to `$HERDR_PLUGIN_STATE_DIR/last-editor` (normally `~/.local/state/herdr/plugins/open-in/last-editor`, holding the bare editor id) and pre-marked the next time the picker opens. When that file is missing or names an editor that is no longer installed, the first row is marked instead. Saving the choice is best-effort: a failure logs a warning and the launch still happens.

### Editor catalog

| Editor         | CLI on `PATH` | macOS app bundle         | How it opens                                              |
| -------------- | ------------- | ------------------------ | --------------------------------------------------------- |
| Zed            | `zed`         | `Zed.app`                | window (`zed -n`)                                         |
| VS Code        | `code`        | `Visual Studio Code.app` | window (`code -n`)                                        |
| Cursor         | `cursor`      | `Cursor.app`             | window (`cursor -n`)                                      |
| Windsurf       | `windsurf`    | `Windsurf.app`           | window (`windsurf -n`)                                    |
| IntelliJ IDEA  | `idea`        | `IntelliJ IDEA.app`      | window (`idea`)                                           |
| WebStorm       | `webstorm`    | `WebStorm.app`           | window (`webstorm`)                                       |
| PyCharm        | `pycharm`     | `PyCharm.app`            | window (`pycharm`)                                        |
| Android Studio | `studio`      | `Android Studio.app`     | window (`studio`, or `open -n -a` when the CLI is absent) |
| nvim           | `nvim`        | —                        | new herdr tab                                             |
| vim            | `vim`         | —                        | new herdr tab                                             |
| helix          | `hx`          | —                        | new herdr tab                                             |

Detection is a CLI probe on `PATH` plus, on macOS, a bundle lookup under `/Applications` and `~/Applications`. A GUI editor without a runnable CLI, for example Android Studio, is still offered when its bundle is installed.

## Development

Development happens against a locally linked plugin; no GitHub install needed.

```
make build                       # ./target/release/herdr-open-in
herdr plugin link .              # register the working directory as a plugin
herdr plugin action invoke open-in.open
herdr plugin log list --plugin open-in
```

`plugin link` does not run build commands, so rebuild after every source change. `make check` runs `cargo fmt --check` and clippy, the prettier Markdown check, `cargo test --locked` and the release build. See [docs/development.md](docs/development.md) for the full dev loop and [CONTRIBUTING.md](CONTRIBUTING.md) for what to run before pushing.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

[MIT](LICENSE)
