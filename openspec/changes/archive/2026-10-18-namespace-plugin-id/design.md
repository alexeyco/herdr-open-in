# Design — namespace-plugin-id

## Approach

A mechanical rename plus a three-file version bump; no control flow moves.
herdr derives the qualified action id itself (`qualified_id()` = plugin id

- `.` + action id), so the only places the plugin hardcodes its herdr
  identity are the manifest `id` and the `PLUGIN_ID` constant used in
  `--plugin` argv. Everything else is prose, tests, or metadata. The one
  substantive spec edit is a correction, not a behavior change: the dispatch
  scenario in `plugin-registration` claimed
  `HERDR_PLUGIN_ACTION_ID=open-in.open`; herdr 0.9.5 passes the bare action
  id `open` (src/main.rs `mode_from` tests already assert against `"open"`),
  and src/main.rs only checks the variable's presence, never its value — so
  the rename cannot affect dispatch.

## Files touched

| File                                         | Edit                                                                                                                                                                                                                            |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `herdr-plugin.toml`                          | `:2` id → `alexeyco.open-in`; `:4` version → `0.4.0`. Name, panes (`picker`, `editor`), action (`open`), build command untouched.                                                                                               |
| `Cargo.toml`                                 | `:3` version → `0.4.0`. Package name `herdr-open-in` untouched.                                                                                                                                                                 |
| `Cargo.lock`                                 | `herdr-open-in` package entry version → `0.4.0` (refreshed by cargo after the Cargo.toml edit).                                                                                                                                 |
| `src/tab.rs`                                 | `:21` `PLUGIN_ID` → `"alexeyco.open-in"`; test argv literals `:197`, `:250`.                                                                                                                                                    |
| `src/main.rs`                                | `:3` doc comment and `:85` unbound-invocation hint → `alexeyco.open-in.open`. The `open-in: ` stderr prefix at `:52`, `:127` stays.                                                                                             |
| `README.md`                                  | `:36` keybinding example, `:43`/`:73` invoke commands, `:74` log-list `--plugin`; new `### Upgrading from 0.3.x` subsection at the end of `## Install`, before `## Usage`. Title `# open-in` (`:1`) and banner alt (`:3`) stay. |
| `docs/how-it-works.md`                       | `:11` (only the `` `open-in.open` `` bind hint on that line; the `open-in:` prefix in the same sentence stays), `:102`, `:127`, `:140`, `:144`.                                                                                 |
| `docs/development.md`                        | `:10`, `:11`, `:12`, `:15` (incl. `unlink open-in` → `unlink alexeyco.open-in`).                                                                                                                                                |
| `AGENTS.md`                                  | `:9`, `:15`; `:15` reworded to "the qualified action id is `alexeyco.open-in.open`" (the local action id remains `open`).                                                                                                       |
| `.github/ISSUE_TEMPLATE/bug_report.yml`      | `:17` log-list `--plugin`.                                                                                                                                                                                                      |
| `CHANGELOG.md`                               | New `## 0.4.0` section above `## 0.3.0`; historical entries at `:26`, `:33` stay verbatim.                                                                                                                                      |
| `openspec/specs/plugin-registration/spec.md` | At sync/archive time: deltas merge by header; Purpose `:5` ("the open-in plugin") and `:9` (qualified action) updated by hand — deltas cannot touch Purpose.                                                                    |

## Decisions

- **Version 0.4.0, not 1.0.0 or 0.3.1.** The rename breaks every existing
  keybinding, which under pre-1.0 semver is a minor bump; 0.3.1 would
  understate it. Release tag `v0.4.0`; the release workflow cross-checks
  the tag against Cargo.toml and herdr-plugin.toml, hence the three-file
  sync in one commit (docs/development.md:28-36).
- **No compatibility alias, no migration.** herdr has no alias mechanism
  for plugin ids, and config/state dirs are keyed by id
  (`herdr plugin config-dir`), so the old `open-in` dirs simply orphan.
  User accepted: last-choice state restarts empty, upgrade steps are
  documented instead.
- **Local ids stay dot-free.** herdr forbids dots in pane and action ids,
  so panes remain `picker`/`editor` and the action remains `open`; only
  the plugin id carries the namespace.
- **`open-in: ` stderr prefix stays.** It is the stable error contract
  quoted by seven baseline specs and explicitly out of scope; renaming it
  would churn every error-path spec for no user gain.
- **Prose uses of "open-in" as the project's colloquial name stay**
  (README title `:1`, banner alt `:3`, docs/how-it-works.md `:3`,
  openspec/specs/editor-catalog/spec.md `:5`): they name the program, not
  the herdr identifier. The repo, crate, and binary keep the
  `herdr-open-in` name regardless.
- **Historical records are never rewritten**: CHANGELOG 0.1.0-0.3.0
  entries and TASK.md keep the old id.
