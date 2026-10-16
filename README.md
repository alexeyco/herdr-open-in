# open-in

![open-in](assets/banner.svg)

herdr plugin that opens the current workspace in the editor you pick.

[herdr](https://herdr.dev) is a terminal workspace manager. This plugin bridges it to the editors already installed on your machine: one keypress opens a picker listing the most popular supported editors detected here, and the choice opens the current workspace directory — a GUI editor in its own window as a detached process, or nvim, vim and helix in a new herdr tab in the current workspace. It targets the herdr plugin API v1 and runs on Linux and macOS (Windows untested, and no Windows build is published).

Other herdr plugins:

- [herdr-revdiff](https://github.com/alexeyco/herdr-revdiff) — review diffs with the revdiff TUI and deliver the resulting annotations to the agent in the focused pane.
- [herdr-gitui](https://github.com/alexeyco/herdr-gitui) — open gitui in a new tab and report what the session changed to the agent in the focused pane.

## Requirements

- herdr >= 0.9.0
- Rust toolchain (the plugin builds itself on install)
- At least one supported editor installed — its CLI on `PATH` or, on macOS, its app bundle in `/Applications` or `~/Applications` (see the [editor catalog](#editor-catalog)); detection probes the CLI with `--version`

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

The popup lists the editors detected on this machine: `↑`/`↓` (or `k`/`j`) move the selection, Enter opens the chosen editor on the workspace directory, Escape or `q` cancels. The last editor you chose is remembered and pre-selected the next time.

### Editor catalog

| Editor         | CLI on `PATH` | macOS app bundle         | How it opens                                              |
| -------------- | ------------- | ------------------------ | --------------------------------------------------------- |
| Android Studio | `studio`      | `Android Studio.app`     | window (`studio`, or `open -n -a` when the CLI is absent) |
| Cursor         | `cursor`      | `Cursor.app`             | window (`cursor -n`)                                      |
| helix          | `hx`          | —                        | new herdr tab                                             |
| IntelliJ IDEA  | `idea`        | `IntelliJ IDEA.app`      | window (`idea`)                                           |
| nvim           | `nvim`        | —                        | new herdr tab                                             |
| PyCharm        | `pycharm`     | `PyCharm.app`            | window (`pycharm`)                                        |
| vim            | `vim`         | —                        | new herdr tab                                             |
| VS Code        | `code`        | `Visual Studio Code.app` | window (`code -n`)                                        |
| WebStorm       | `webstorm`    | `WebStorm.app`           | window (`webstorm`)                                       |
| Windsurf       | `windsurf`    | `Windsurf.app`           | window (`windsurf -n`)                                    |
| Zed            | `zed`         | `Zed.app`                | window (`zed -n`)                                         |

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
