# editor-launch Specification

## Purpose

Editor launch covers what happens after the picker returns a choice: a GUI
editor row is started on the resolved workspace directory as a detached
process that outlives the picker popup, while a terminal editor row is handed
to the separate herdr-tab capability (`tab::open_terminal_tab`, owned by
another specification and not restated here). The GUI strategy prefers a real
CLI — the bare command and its catalog aliases probed on `PATH`, then known
per-OS absolute install locations — and falls back on macOS to opening the
installed `.app` bundle through `open -n -a` (src/launch.rs). Every launch is
fire-and-forget: the picker process spawns the child, does not wait for it,
and exits, relying on a fresh child process group so the `SIGHUP` sent when
the popup pty is torn down does not kill the editor.

## Requirements

### Requirement: Dispatch by editor kind

After a successful picker selection the plugin SHALL launch a `Gui` row
through the detached GUI path and a `Terminal` row through the herdr tab
request, choosing by the row's kind with no other branching
(src/main.rs:133-138; terminal rows per src/editors.rs:135-164).

#### Scenario: GUI row detaches

- **WHEN** the user selects a catalog row whose kind is GUI
- **THEN** the plugin locates and spawns its launcher as a detached process
  and returns without waiting for the editor to exit

#### Scenario: Terminal row opens a tab

- **WHEN** the user selects a catalog row whose kind is Terminal
- **THEN** the plugin issues the herdr tab request instead of spawning a
  detached GUI process (mechanics owned by the tab capability)

### Requirement: CLI discovery order

To launch a GUI row the plugin SHALL first probe the row's command names — the
bare `cmd` then each entry of `alts`, in order — on `PATH`, and only when none
probes true SHALL it probe the known absolute install locations: on macOS
`/usr/local/bin/<name>` for every command name plus the row's inner bundle CLI
at `/Applications/<stem>.app/<bundle_cli>`; on Linux `<dir>/<name>` for every
command name in each of `/usr/bin`, `/usr/snap/bin` and `/snap/bin`, in that
directory order; on other operating systems there are no absolute fallbacks
(src/launch.rs:23-55, 150-182; tests src/launch.rs:266-317). A probe SHALL run
`<candidate> --version` with both streams discarded and SHALL count the
candidate as found when it runs and exits with status 0
(src/launch.rs:139-148).

#### Scenario: Bare command on PATH wins

- **WHEN** the row's bare command answers `<cmd> --version` with exit 0 on
  `PATH`
- **THEN** that command is launched and no absolute fallback location is
  probed

#### Scenario: Alias found on PATH

- **WHEN** the bare command is absent but an alias (for example `zeditor` for
  Zed) probes true on `PATH`
- **THEN** the alias is used as the launch command

#### Scenario: Absolute fallback on Linux

- **WHEN** on Linux no command name probes true on `PATH` but
  `/snap/bin/<name>` probes true
- **THEN** `/snap/bin/<name>` is used as the launch command

#### Scenario: Probe output is discarded

- **WHEN** a probe runs
- **THEN** the child's stdout and stderr go to the null device and only its
  exit status is observed

### Requirement: argv and inherited context

A CLI launch SHALL execute the chosen command with the row's argv template
from the catalog, substituting the single `<dir>` placeholder with the
resolved workspace directory; the directory SHALL travel only as an argument.
The plugin SHALL NOT alter the child's environment or working directory: the
detached command builder sets only arguments, standard streams and (on unix)
the process group, so the child inherits the picker process environment and
cwd unchanged (src/launch.rs:57-70, 97-127 — no `.env*` or `.current_dir`
calls; tests src/launch.rs:228-251).

#### Scenario: Placeholder substituted once

- **WHEN** a row with args `["-n", "<dir>"]` is launched on directory `/work/demo`
- **THEN** the spawned argv is `["-n", "/work/demo"]`, with the directory
  appearing exactly once across every catalog row

### Requirement: macOS app-bundle fallback

When no CLI is found for a GUI row, the plugin SHALL fall back — only if the
row declares a bundle stem and that `<stem>.app` directory exists under
`/Applications` or, when `HOME` is set, under `$HOME/Applications` — to
spawning `open -n -a "<full bundle path>" <dir>`, opening a new instance of
the app on the directory (src/launch.rs:72-95, 184-198; test
src/launch.rs:254-264). On operating systems other than macOS the bundle
lookup never matches and only the CLI path can succeed (src/launch.rs:200-204).

#### Scenario: Bundle opened when no CLI exists

- **WHEN** on macOS a row such as Android Studio has no probeable CLI but its
  `.app` directory exists under `/Applications`
- **THEN** the plugin spawns `open -n -a "/Applications/<stem>.app" <dir>`

#### Scenario: No bundle directory means no fallback

- **WHEN** no CLI probes true and the row's `.app` directory is absent from
  both application directories
- **THEN** the bundle fallback is not attempted and the launch fails with the
  missing-launcher error of the "Launch failure reporting" requirement below,
  naming the row's display name and bare command (`src/launch.rs:208-213`)

### Requirement: Detached spawn

The GUI launch SHALL spawn the child and return as soon as the spawn
succeeds, never waiting for its exit status; the child's stdin, stdout and
stderr SHALL all be the null device; on unix the child SHALL be placed in its
own process group via `process_group(0)` — std's safe equivalent of the
`setpgid` half of `setsid`, with no `setsid` and no `unsafe` — so the `SIGHUP`
the pty delivers to the popup's foreground group on popup exit does not reach
the child; on non-unix builds no process-group call is made
(src/launch.rs:97-137; test src/launch.rs:337-369).

#### Scenario: Editor survives popup exit

- **WHEN** a GUI editor has been spawned and the picker popup then exits,
  tearing down its pty
- **THEN** the editor, being in a process group of its own led by its own pid,
  is not signalled and keeps running

#### Scenario: No waiting

- **WHEN** the spawn succeeds
- **THEN** `launch_gui` returns `Ok` immediately without reading the child's
  output or waiting for it

### Requirement: Launch failure reporting

When a GUI row has neither a probeable CLI nor an installed bundle, the launch
SHALL fail with an error naming the row's display name and bare command, of
the form `cannot launch <Name>: no \`<cmd>\` CLI found on PATH or at the known
install locations`(src/launch.rs:36-44, 206-213; test src/launch.rs:319-324).
When the spawn itself fails, the launch SHALL fail with an error naming the
display name, the program, and the OS error, of the form`failed to launch
<Name> with <program>: <error>`(src/launch.rs:129-137). In both cases the
plugin's main prints the message on stderr prefixed with`open-in: ` and
exits with status 1
(src/main.rs:49-53). Persistence of the chosen id is best-effort and runs
before the launch, as described in the last-choice-state capability
(src/main.rs:123-129).

#### Scenario: Missing launcher

- **WHEN** neither a CLI nor a bundle can be located for the chosen GUI row
- **THEN** stderr shows `open-in: cannot launch <Name>: no \`<cmd>\` CLI found
  on PATH or at the known install locations` and the process exits 1

#### Scenario: Spawn error surfaces

- **WHEN** the located program cannot be spawned
- **THEN** stderr shows `open-in: failed to launch <Name> with <program>:
<error>` and the process exits 1
