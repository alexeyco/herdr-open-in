# Namespace the plugin id to `alexeyco.open-in`

## Why now

The plugin registers with herdr under the bare id `open-in`
(herdr-plugin.toml:2), while the author-namespaced convention is what herdr
plugins actually use — `alexeyco.review` and `ray.plugin-manager` sit
installed alongside it — and a bare id reads as off-rules. Nothing forces
the bare form: herdr 0.9.5 accepts plugin ids of letters, digits, `.`, `:`,
`_`, `-` up to 120 characters, so dots are legal (only local pane and
action ids forbid them). The baseline spec additionally misstates the
runtime contract: openspec/specs/plugin-registration/spec.md:96 claims
`HERDR_PLUGIN_ACTION_ID` carries the qualified `open-in.open`, while herdr
passes the bare action id `open` — the `mode_from` tests in src/main.rs
already use that value. This change renames the id everywhere, corrects
that spec text, and ships the result as 0.4.0 with upgrade instructions
from 0.3.x.

## What changes

- **Plugin id.** `open-in` → `alexeyco.open-in` in the manifest
  (herdr-plugin.toml:2) and in the only runtime hardcode, `PLUGIN_ID` in
  src/tab.rs:21, which feeds the `--plugin` argv of both pane-open calls
  (src/tab.rs:82, :107) and their test assertions (src/tab.rs:197, :250).
- **Qualified action id.** The action becomes `alexeyco.open-in.open`
  (herdr's `qualified_id()` is plugin id + `.` + action id). The local
  action id stays `open` and the pane ids stay `picker`/`editor` — herdr
  forbids dots in local pane/action ids.
- **Unbound-invocation error text.** The hint in src/main.rs:85 (and the
  module doc comment at src/main.rs:3) now names
  `alexeyco.open-in.open`. The stderr prefix `open-in: ` is untouched.
- **Version 0.3.0 → 0.4.0** in Cargo.toml:3, the `herdr-open-in` entry of
  Cargo.lock, and herdr-plugin.toml:4 — the three-file sync rule of
  docs/development.md:28-36; the release tag will be `v0.4.0`.
- **Upgrade path documented.** A README subsection and the CHANGELOG
  `## 0.4.0` entry tell users to `herdr plugin uninstall open-in`,
  `herdr plugin install alexeyco/herdr-open-in`, rebind their key to
  `alexeyco.open-in.open`, and expect the last-chosen editor to start
  empty (herdr keys config/state dirs by plugin id; the old `open-in`
  dirs orphan and are not migrated).
- **Docs and repo metadata.** README.md:36,43,73,74;
  docs/how-it-works.md:11,102,127,140,144; docs/development.md:10-12,15;
  AGENTS.md:9,15; .github/ISSUE_TEMPLATE/bug_report.yml:17.
- **Spec correction.** The plugin-registration dispatch scenario
  "Entrypoint present alongside action id" states
  `HERDR_PLUGIN_ACTION_ID=open` (bare), matching observed herdr 0.9.5
  behavior; the version literals in the identity requirement become
  `0.4.0`.

Not changing: the stderr error prefix `open-in: ` (stable contract cited
by seven baseline specs), the crate/binary name `herdr-open-in`, the
repository name, historical CHANGELOG.md and TASK.md entries, the user's
local `~/.config/herdr/config.toml` keybinding, and any migration of old
`open-in` config/state directories.

## Impact on capabilities

| Capability                                                                                                                               | Impact                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| ---------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `plugin-registration`                                                                                                                    | MODIFIED "Manifest declares plugin identity and build" (id, version 0.4.0), "Manifest declares the open action" (qualified id `alexeyco.open-in.open`), "Entrypoint variable wins over action variable in dispatch" (scenario corrected to the bare `HERDR_PLUGIN_ACTION_ID=open`), "Unknown entrypoint and unbound invocation fail with named errors" (error text names the new qualified id). Pane declarations and the error/exit convention are untouched. |
| `herdr-pane-control`                                                                                                                     | MODIFIED "Terminal-tab pane open argv" and "Picker popup pane open argv": the `--plugin` argv literal becomes `alexeyco.open-in`. Resolution, pane-id parsing, and failure formatting untouched.                                                                                                                                                                                                                                                               |
| `editor-catalog`, `editor-detection`, `editor-launch`, `editor-picker`, `last-choice-state`, `terminal-editor-exec`, `workspace-context` | Unchanged. Dispatch never inspects the action id's value (src/main.rs checks presence only), and last-choice state keeps working through `$HERDR_PLUGIN_STATE_DIR` — which herdr now points at a fresh, empty directory (accepted).                                                                                                                                                                                                                            |

## Assumptions and accepted risks

ASSUMPTION: the author handle is `alexeyco` (matches the GitHub owner `alexeyco/herdr-open-in`); the user confirmed the resulting plugin id `alexeyco.open-in`.
ASSUMPTION: `min_herdr_version` stays `0.9.0` (untested — no read-only way to exercise id validation on herdr <0.9.5; `herdr plugin link --help → Link a local plugin … <PATH>` shows no dry-run flag. A herdr older than 0.9.5 rejecting dots in plugin ids would fail link/install on the manifest id and force raising `min_herdr_version` to `0.9.5`).
REJECTED-RISK: herdr config/state dirs are keyed by plugin id, so after the rename `~/.config/herdr/plugins/config/open-in` and the matching state dir orphan and the last-chosen editor restarts empty; the user declined any migration.
REJECTED-RISK: existing 0.3.x keybindings bound to `open-in.open` stop working after the upgrade; the user accepted the breakage (minor bump 0.4.0, documented upgrade steps) instead of a compatibility alias.
