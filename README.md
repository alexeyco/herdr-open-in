# open-in

![open-in](assets/banner.svg)

herdr plugin that opens the current workspace in the editor you pick.

[herdr](https://herdr.dev) is a terminal workspace manager. This plugin bridges it to the editors already installed on your machine: one detected editor opens directly; when several are available, a compact picker lets you choose one. The choice opens the current workspace directory — a GUI editor in its own window as a detached process, or nvim, vim and helix in a new herdr tab in the current workspace. It targets the herdr plugin API v1 and runs on Linux and macOS (Windows untested, and no Windows build is published).

Other herdr plugins:

- [herdr-git](https://github.com/alexeyco/herdr-git) — open gitui in a new tab and report what the session changed to the agent in the focused pane.
- [herdr-review](https://github.com/alexeyco/herdr-review) — review diffs with the human review TUI of your choice and deliver your annotations to the agent in the focused pane.

## Requirements

- herdr >= 0.9.0
- Rust toolchain (the plugin builds itself on install)
- At least one supported editor installed — its CLI on `PATH` or, on macOS, its app bundle in `/Applications` or `~/Applications` (see the [editor catalog](#editor-catalog)); detection probes the CLI with `--version`

Zed's CLI is installed from Zed's command palette: run `cli: install cli binary`. See the [Zed CLI reference](https://zed.dev/docs/reference/cli).

## Install

```
herdr plugin install alexeyco/herdr-open-in
```

### Upgrading from 0.3.x

The plugin id is renamed from `open-in` to `alexeyco.open-in`. Uninstall the old plugin, then install the new one:

```
herdr plugin uninstall open-in
herdr plugin install alexeyco/herdr-open-in
```

Rebind the key from `open-in.open` to `alexeyco.open-in.open` in your herdr config. Config and state under the old id are not migrated — the last-chosen editor starts empty.

## Usage

Bind a key to the action in your herdr config:

```toml
[[keys.command]]
key = "prefix+shift+o"
type = "plugin_action"
command = "alexeyco.open-in.open"
description = "Open workspace in an editor"
```

Any chord works; see the [herdr keyboard docs](https://herdr.dev/docs/keyboard/) for picking one that your terminal doesn't swallow. You can also invoke the action from the CLI:

```
herdr plugin action invoke alexeyco.open-in.open
```

When more than one editor is detected, the compact popup lists them in order and marks the selected row with `›`. Use `↑` and `↓` to move, Enter to open the selected editor on the workspace directory, or Escape to cancel. Other keys have no effect. The last editor you chose is remembered and pre-selected the next time; a sole detected editor opens directly without the picker.

## Comparison

How this plugin compares with the five other editor-launch tools that integrate with herdr, cell by cell as documented in their public sources.

✅ documented in the linked sources · ⚠️ partial or implemented differently · ❌ not found in the public sources checked

| Criterion                              | [herdr-open-in](https://github.com/alexeyco/herdr-open-in) | [timofey-TK](https://github.com/timofey-TK/herdr-open-in-editor) | [open-project-with](https://github.com/harish-zuper/herdr-open-project-with) | [ide-jump](https://github.com/agentience/herdr-plugin-ide-jump) | [cursor-open](https://github.com/alex-devdone/herdr-cursor-open) | [upnorthmedia](https://github.com/upnorthmedia/herdr-open-in-editor) |
| -------------------------------------- | ---------------------------------------------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------- |
| Opens the workspace via a herdr action | ✅                                                         | ✅ [^1]                                                          | ✅ [^2]                                                                      | ✅ [^3]                                                         | ✅ [^4]                                                          | ✅ [^5]                                                              |
| Picker of choices at one keypress      | ✅                                                         | ❌ [^6]                                                          | ✅ [^7]                                                                      | ⚠️ [^8]                                                         | ✅ [^9]                                                          | ❌ [^10]                                                             |
| Editors discovered, not preconfigured  | ✅                                                         | ⚠️ [^11]                                                         | ✅ [^12]                                                                     | ❌ [^13]                                                        | ❌ [^14]                                                         | ⚠️ [^15]                                                             |
| Terminal editors run inside herdr      | ✅                                                         | ❌                                                               | ❌                                                                           | ❌                                                              | ❌                                                               | ❌                                                                   |
| JetBrains-family coverage              | ✅                                                         | ❌                                                               | ✅ [^16]                                                                     | ❌ [^17]                                                        | ❌                                                               | ⚠️ [^18]                                                             |
| Remembers the last choice              | ✅                                                         | ❌                                                               | ❌                                                                           | ⚠️ [^19]                                                        | ❌                                                               | ❌                                                                   |
| Runs on both Linux and macOS           | ✅                                                         | ✅ [^20]                                                         | ❌ [^21]                                                                     | ❌ [^22]                                                        | ❌ [^23]                                                         | ❌ [^24]                                                             |

[^1]: timofey-TK — installs as a herdr plugin; its actions open the workspace in VS Code or Zed — [source](https://github.com/timofey-TK/herdr-open-in-editor).

[^2]: harish-zuper — the herdr action opens the project in the app picked from the fzf popup (Finder, VS Code, IDEA, Android Studio, Xcode, Terminal) — [source](https://github.com/harish-zuper/herdr-open-project-with).

[^3]: agentience — the herdr action surfaces the workspace in an IDE by raising an already-open IDE window rather than launching an editor — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^4]: alex-devdone — the herdr action opens the workspace in Cursor or VS Code — [source](https://github.com/alex-devdone/herdr-cursor-open).

[^5]: upnorthmedia — the herdr action opens the workspace, or a file:line path the agent printed, in the configured editor — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

[^6]: timofey-TK — no picker: the plugin picks automatically by preference and ships explicit per-editor actions (`open-vscode`, `open-zed`) — [source](https://github.com/timofey-TK/herdr-open-in-editor).

[^7]: harish-zuper — one keypress opens an fzf fuzzy popup over the apps discovered at popup open — [source](https://github.com/harish-zuper/herdr-open-project-with).

[^8]: agentience — the popup lists already-open IDE windows, preselecting the project match: windows to raise, not editors to launch — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^9]: alex-devdone — a popup offers the configured editor set under the keys 1 and 2 — [source](https://github.com/alex-devdone/herdr-cursor-open).

[^10]: upnorthmedia — `editor.conf` fixes one editor; the README states "no picker and nothing to type" — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

[^11]: timofey-TK — probes the CLI on `PATH` only, with no macOS app-bundle fallback: a GUI editor without a runnable CLI is not offered — [source](https://github.com/timofey-TK/herdr-open-in-editor).

[^12]: harish-zuper — discovers the installed `.app` bundles when the popup opens — [source](https://github.com/harish-zuper/herdr-open-project-with).

[^13]: agentience — `config.json` names the app to raise; nothing is discovered from the machine — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^14]: alex-devdone — the editor set is configured, not discovered — [source](https://github.com/alex-devdone/herdr-cursor-open).

[^15]: upnorthmedia — probes `PATH` then `/Applications`, but only within the preconfigured editor set (Zed, VS Code, Cursor, Sublime, custom template) — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

[^16]: harish-zuper — the discovered set includes IntelliJ IDEA and Android Studio — [source](https://github.com/harish-zuper/herdr-open-project-with).

[^17]: agentience — no JetBrains launch: an IDE is offered only when its window is already open and named in `config.json` — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^18]: upnorthmedia — no built-in JetBrains rows; a custom editor template can name any CLI — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

[^19]: agentience — preselects the window matching the current project: a project match, not a remembered last choice — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^20]: timofey-TK — documented for Linux and macOS — [source](https://github.com/timofey-TK/herdr-open-in-editor).

[^21]: harish-zuper — documented as macOS-only (`.app` bundle discovery) — [source](https://github.com/harish-zuper/herdr-open-project-with).

[^22]: agentience — documented for macOS and Windows; no Linux — [source](https://github.com/agentience/herdr-plugin-ide-jump).

[^23]: alex-devdone — documented as macOS-only, implemented in bash — [source](https://github.com/alex-devdone/herdr-cursor-open).

[^24]: upnorthmedia — documented as macOS-only — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

All cells checked 2026-10-22 against the linked public sources.

<details>
<summary>Detailed comparison</summary>

|                               | [herdr-open-in](https://github.com/alexeyco/herdr-open-in)                                               | [timofey-TK](https://github.com/timofey-TK/herdr-open-in-editor)                                                      | [open-project-with](https://github.com/harish-zuper/herdr-open-project-with)                         | [ide-jump](https://github.com/agentience/herdr-plugin-ide-jump)                                                    | [cursor-open](https://github.com/alex-devdone/herdr-cursor-open)                                   | [upnorthmedia](https://github.com/upnorthmedia/herdr-open-in-editor)                                                             |
| ----------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| Editor selection              | popup picker of the editors detected on this machine ([repo](https://github.com/alexeyco/herdr-open-in)) | automatic by preference plus explicit per-editor actions ([repo](https://github.com/timofey-TK/herdr-open-in-editor)) | fzf fuzzy popup of discovered apps ([repo](https://github.com/harish-zuper/herdr-open-project-with)) | popup of open IDE windows, project match preselected ([repo](https://github.com/agentience/herdr-plugin-ide-jump)) | popup keys 1/2 over the configured set ([repo](https://github.com/alex-devdone/herdr-cursor-open)) | fixed editor from `editor.conf` ("no picker and nothing to type") ([repo](https://github.com/upnorthmedia/herdr-open-in-editor)) |
| External runtime dependencies | none beyond herdr (Rust binary) ([repo](https://github.com/alexeyco/herdr-open-in))                      | python3 ([repo](https://github.com/timofey-TK/herdr-open-in-editor))                                                  | fzf ([repo](https://github.com/harish-zuper/herdr-open-project-with))                                | python3 ([repo](https://github.com/agentience/herdr-plugin-ide-jump))                                              | none (bash) ([repo](https://github.com/alex-devdone/herdr-cursor-open))                            | jq ([repo](https://github.com/upnorthmedia/herdr-open-in-editor))                                                                |
| Platforms                     | Linux, macOS ([repo](https://github.com/alexeyco/herdr-open-in))                                         | Linux, macOS ([repo](https://github.com/timofey-TK/herdr-open-in-editor))                                             | macOS ([repo](https://github.com/harish-zuper/herdr-open-project-with))                              | macOS, Windows ([repo](https://github.com/agentience/herdr-plugin-ide-jump))                                       | macOS ([repo](https://github.com/alex-devdone/herdr-cursor-open))                                  | macOS ([repo](https://github.com/upnorthmedia/herdr-open-in-editor))                                                             |
| Latest release                | [releases](https://github.com/alexeyco/herdr-open-in/releases)                                           | [no releases](https://github.com/timofey-TK/herdr-open-in-editor/releases)                                            | [v0.1.0](https://github.com/harish-zuper/herdr-open-project-with/releases/tag/v0.1.0)                | [no releases](https://github.com/agentience/herdr-plugin-ide-jump/releases)                                        | [no releases](https://github.com/alex-devdone/herdr-cursor-open/releases)                          | [no releases](https://github.com/upnorthmedia/herdr-open-in-editor/releases)                                                     |
| License / language            | [MIT · Rust](https://github.com/alexeyco/herdr-open-in)                                                  | [MIT · Python](https://github.com/timofey-TK/herdr-open-in-editor)                                                    | [MIT · Shell](https://github.com/harish-zuper/herdr-open-project-with)                               | [MIT · Python](https://github.com/agentience/herdr-plugin-ide-jump)                                                | [MIT · Shell](https://github.com/alex-devdone/herdr-cursor-open)                                   | [MIT · Shell](https://github.com/upnorthmedia/herdr-open-in-editor)                                                              |

</details>

Among the tools compared above, this plugin is the only one that combines a picker of the editors discovered on this machine, terminal editors (nvim, vim, helix) running in herdr tabs, the macOS app-bundle fallback for GUI editors whose CLI is missing, and last-choice memory.

Also exists in this space, without a cell-by-cell comparison above:

- [herdr_plugin](https://github.com/Andreslvc/herdr_plugin) — opens VS Code, copies paths, handles spaces in paths.
- [herdr-positron](https://github.com/gadenbuie/herdr-positron) — opens Positron; POSITRON_CMD overrides the binary.
- [herdr-open](https://github.com/kongdd/herdr-open) — two fixed actions for Zed and VS Code; SSH relay; jq dependency.
- [herdr-launcher-pane](https://github.com/y-hirakaw/herdr-launcher-pane) — workspace-by-command dock with a mouse UI; Finder and VS Code built in.
- [hither](https://github.com/T0mSIlver/hither) — remote pane; opens a Zed SSH project on macOS via launchd.

## Editor catalog

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
herdr plugin action invoke alexeyco.open-in.open
herdr plugin log list --plugin alexeyco.open-in
```

`plugin link` does not run build commands, so rebuild after every source change. `make check` runs `cargo fmt --check` and clippy, the prettier Markdown check, `cargo test --locked` and the release build. See [docs/development.md](docs/development.md) for the full dev loop and [CONTRIBUTING.md](CONTRIBUTING.md) for what to run before pushing.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

## License

[MIT](LICENSE)
