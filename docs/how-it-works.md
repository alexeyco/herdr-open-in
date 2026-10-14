# How it works

`open-in` is a small Rust program (sources under `src/`). herdr runs it in one of three modes, and the binary picks the mode from `HERDR_PLUGIN_ENTRYPOINT_ID` (which wins) or `HERDR_PLUGIN_ACTION_ID`:

| Mode   | Environment set by herdr            | What runs                                                        |
| ------ | ----------------------------------- | ---------------------------------------------------------------- |
| Picker | `HERDR_PLUGIN_ENTRYPOINT_ID=picker` | the popup dialog; launches whatever the user picks               |
| Editor | `HERDR_PLUGIN_ENTRYPOINT_ID=editor` | the chosen terminal editor in the foreground, inside its own tab |
| Action | only `HERDR_PLUGIN_ACTION_ID`       | opens the picker popup and exits                                 |

An entrypoint other than `picker` or `editor` is an error naming it, and with neither variable set the binary exits 1 with a hint to bind `open-in.open` in herdr. Every error is one line on stderr prefixed `open-in:` and exits 1.

## Context

Environment variables set by herdr (see the [plugin docs](https://herdr.dev/docs/plugins/)):

| Variable                               | Used for                                                                       |
| -------------------------------------- | ------------------------------------------------------------------------------ |
| `HERDR_PLUGIN_CONTEXT_JSON`            | Source of the directory and the `workspace_id`; must be present                |
| `HERDR_PLUGIN_ENTRYPOINT_ID`           | Selects the `picker` or `editor` mode; wins over the action id                 |
| `HERDR_PLUGIN_ACTION_ID`               | Selects action mode when no entrypoint is set: open the picker popup           |
| `HERDR_BIN_PATH`                       | Absolute path of the herdr binary used to open panes; never assumed to be bare |
| `HERDR_PLUGIN_STATE_DIR`               | Directory of the `last-editor` file                                            |
| `HERDR_PLUGIN_ID`, `HERDR_PLUGIN_ROOT` | Set by herdr but unused by this plugin (no herdr callbacks needed)             |

A popup process carries no `HERDR_WORKSPACE_ID`, `HERDR_TAB_ID` or `HERDR_PANE_ID`, only the context JSON, so the `workspace_id` is read from that JSON and passed to herdr explicitly.

Fields read from the context JSON:

| Field                    | Meaning                                                      |
| ------------------------ | ------------------------------------------------------------ |
| `worktree.checkout_path` | Checkout directory of the worktree the workspace is based on |
| `workspace_cwd`          | Workspace working directory (may be a subdirectory)          |
| `focused_pane_cwd`       | Working directory of the focused pane                        |
| `workspace_id`           | Workspace to open the terminal tab in; must be present       |

## Path resolution

```
worktree.checkout_path > workspace_cwd > focused_pane_cwd > error
```

`worktree.checkout_path` wins because `workspace_cwd` may be a subdirectory the user has `cd`-ed into; opening the editor there would show the subdirectory as the project root instead of the repository. A field counts only if it is a present, non-blank string; a non-string or whitespace-only value counts as absent and the chain falls through to the next candidate.

If no field yields a directory, the action exits 1 with `no directory in plugin context`. This happens when the action is invoked outside a workspace context. `workspace_id` is required separately: a missing, non-string or blank value is `no workspace_id in plugin context`.

## Editor catalog

The picker offers a fixed catalog, in this order, and detection preserves it:

| id               | Name           | Kind     | CLI probed | argv       | macOS bundle                                                 |
| ---------------- | -------------- | -------- | ---------- | ---------- | ------------------------------------------------------------ |
| `zed`            | Zed            | GUI      | `zed`      | `-n <dir>` | `Zed.app` (`Contents/MacOS/cli`)                             |
| `code`           | VS Code        | GUI      | `code`     | `-n <dir>` | `Visual Studio Code.app` (`Contents/Resources/app/bin/code`) |
| `cursor`         | Cursor         | GUI      | `cursor`   | `-n <dir>` | `Cursor.app` (`Contents/Resources/app/bin/cursor`)           |
| `windsurf`       | Windsurf       | GUI      | `windsurf` | `-n <dir>` | `Windsurf.app` (`Contents/Resources/app/bin/windsurf`)       |
| `idea`           | IntelliJ IDEA  | GUI      | `idea`     | `<dir>`    | `IntelliJ IDEA.app` (`Contents/MacOS/idea`)                  |
| `webstorm`       | WebStorm       | GUI      | `webstorm` | `<dir>`    | `WebStorm.app` (`Contents/MacOS/webstorm`)                   |
| `pycharm`        | PyCharm        | GUI      | `pycharm`  | `<dir>`    | `PyCharm.app` (`Contents/MacOS/pycharm`)                     |
| `android-studio` | Android Studio | GUI      | `studio`   | `<dir>`    | `Android Studio.app` (`Contents/MacOS/studio`)               |
| `nvim`           | nvim           | terminal | `nvim`     | `<dir>`    | none                                                         |
| `vim`            | vim            | terminal | `vim`      | `<dir>`    | none                                                         |
| `helix`          | helix          | terminal | `hx`       | `<dir>`    | none                                                         |

`<dir>` is the directory placeholder replaced by the resolved path at launch time.

## Detection

An editor counts as installed when its CLI responds on `PATH` (probed with `<cmd> --version` with both streams discarded; exit status 0 wins), or, on macOS, when its app bundle sits under `/Applications` or `~/Applications`. A bundle is accepted only when its `.app` directory exists and, where the row records an inner CLI, that inner CLI exists or is executable inside it, so a leftover empty bundle directory is not mistaken for a working install. Linux and Windows have no bundles: the CLI probe is the only source.

Detection runs when the picker starts. A catalog with zero hits prints exactly one error line naming every catalog editor and the install hint, then exits 1:

```
no supported editor was found; install one of: Zed, VS Code, Cursor, Windsurf, IntelliJ IDEA, WebStorm, PyCharm, Android Studio, nvim, vim, helix (its CLI on PATH or, on macOS, its app bundle in /Applications or ~/Applications)
```

A single detected editor still shows the dialog.

## GUI editors

A GUI editor is launched on the resolved directory and then left alone: the plugin returns as soon as the process is spawned and never waits for its exit status. All three standard streams of the child are discarded.

The CLI is preferred, looked up in this order:

1. The bare command (`<cmd> --version`) on `PATH`
2. Known absolute locations, per OS:

| OS      | Fallbacks                                                                                  |
| ------- | ------------------------------------------------------------------------------------------ |
| macOS   | `/usr/local/bin/<cmd>`, and `/Applications/<bundle>.app/<inner CLI>` when the row has both |
| Linux   | `/usr/bin/<cmd>`, `/usr/snap/bin/<cmd>`, `/snap/bin/<cmd>`                                 |
| Windows | none (PATH only)                                                                           |

On macOS, a row whose bundle is installed but whose CLI cannot be found falls back to `open -n -a "<bundle>.app" <dir>` (Android Studio ships no `studio` CLI there). If neither a CLI nor an installed bundle is available, the action exits 1 with `cannot launch <Name>: no `<cmd>` CLI found on PATH or at the known install locations`.

## Terminal editors

nvim, vim and helix run inside herdr, so the picker asks herdr for a new tab running this same binary's `editor` entrypoint:

```
$HERDR_BIN_PATH plugin pane open \
  --plugin open-in --entrypoint editor --placement tab \
  --workspace <workspace_id> --cwd <dir> \
  --env OPEN_IN_EDITOR=<id> --env OPEN_IN_DIR=<dir>
```

Plugin pane commands are argv-exec with no shell, and `--env` cannot override the `HERDR_*` variables, hence the custom names. The picker checks `result.plugin_pane.pane.pane_id` in herdr's response and returns; a rejection surfaces herdr's own error text. The `editor` pane process then looks its id up in the catalog and runs the editor in the foreground, owning the tty until it exits, so a non-zero exit becomes `<Name> exited with <status>`.

## The picker dialog

The popup is a tiny raw-mode TUI. Raw mode goes through an `stty` child process only — no FFI, no `unsafe`, no extra dependency. The current settings are saved with `stty -g` and switched with `stty raw -echo min 0 time 1`; the saved settings and the cursor are restored on every exit path. The cursor is hidden while the picker redraws and shown again on the way out.

| Key                                           | Effect                                                               |
| --------------------------------------------- | -------------------------------------------------------------------- |
| Up / Down, `k` / `j` (CSI and SS3 arrows too) | move the selection                                                   |
| Enter                                         | launch the selected editor                                           |
| `q`, Escape                                   | cancel and exit 0 without launching anything; other keys are ignored |

`min 0 time 1` gives every read a 0.1 s timeout instead of blocking, so a lone Escape cancels without waiting for a second byte. Cancel (or running out of input when stdin is not a tty) is a clean exit 0. On non-unix builds the picker is a stub that returns an error.

Action mode (only `HERDR_PLUGIN_ACTION_ID` set) opens this dialog with `$HERDR_BIN_PATH plugin pane open --plugin open-in --entrypoint picker --placement popup --focus` and checks only the exit status; a popup response carries no pane id.

## The last choice

The chosen editor id is written to `$HERDR_PLUGIN_STATE_DIR/last-editor` — the bare id plus a trailing newline — through a sibling temp file and a rename, so a reader never sees a half-written file. The next time the picker opens it pre-marks that row; an id that is no longer in the detected list falls back to the first row. Saving is best-effort: a failure (for example an unset state dir) prints a warning and the launch still happens.

## JSON parsing

The context is parsed by [`serde_json`](https://crates.io/crates/serde_json) into a generic `Value`. The plugin then takes the first present, non-blank string along the resolution chain (`worktree.checkout_path` → `workspace_cwd` → `focused_pane_cwd`) and reads `workspace_id` from the same document. Standard JSON escaping, including `\uXXXX` with surrogate pairs, is handled by serde_json, so Windows paths (`C:\\Users`) and non-ASCII directory names survive.

## Troubleshooting

- **`no supported editor was found`** — none of the catalog CLIs responded and no macOS bundle matched. Install an editor, put its CLI on `PATH`, or install its app in `/Applications` or `~/Applications`, then re-run the action. The picker writes the line to stderr, exits 1, and the popup closes; the line does not appear in `herdr plugin log list` (see the note on logs below).
- **`no directory in plugin context`** — the action ran without workspace context. Invoke it from inside a herdr workspace (e.g. via the keybinding or `herdr plugin action invoke open-in.open`), not from a bare shell.
- **`HERDR_BIN_PATH is not set or blank`** — the binary was started outside herdr, so it cannot open a pane. Invoke the action through herdr.
- **`cannot launch <Name>: no `<cmd>` CLI found`** — a GUI editor was selected but its CLI is not on `PATH` and (on macOS) its bundle is not installed.
- **`<Name> exited with ...`** — a terminal editor launched but returned a non-zero status; run it manually in that directory to see the underlying error.
- **Logs** — `herdr plugin log list --plugin open-in` lists the plugin's action records: the `open-in.open` invocation's own exit status, stdout and stderr. The picker runs as a separate popup pane, so its stderr and exit status are not captured there. To read a picker error line directly, run the built binary from a terminal with `HERDR_PLUGIN_ENTRYPOINT_ID=picker` and a synthetic `HERDR_PLUGIN_CONTEXT_JSON`; errors raised before the dialog starts print to stderr and exit 1. The herdr server log (`herdr --help` prints the path; `~/.config/herdr/herdr-server.log` on macOS) records a pane exit as `pane child exited … status="ExitStatus { code: 1, … }"`.
