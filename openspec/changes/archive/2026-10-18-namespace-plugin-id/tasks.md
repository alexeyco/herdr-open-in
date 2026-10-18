# Tasks — namespace-plugin-id

## 1. Plugin id in manifest and runtime

- [x] 1.1 Set `id = "alexeyco.open-in"` in `herdr-plugin.toml:2`. Leave
      `name`, `min_herdr_version`, `platforms`, the build command, the
      pane ids `picker`/`editor`, and the action id `open` untouched —
      herdr forbids dots in local pane/action ids (scenarios: "Manifest
      fields are readable by herdr", "Action is available in workspace
      context").
- [x] 1.2 Change `const PLUGIN_ID: &str = "open-in";` at `src/tab.rs:21`
      to `"alexeyco.open-in"`; the constant feeds the `--plugin` argv
      element of both `pane_open_argv` (src/tab.rs:82) and
      `picker_popup_argv` (src/tab.rs:107), so no other call site
      changes. In the same task, grep the value's literal uses —
      `grep -n '"open-in"' src/` — and update the two expected-argv test
      literals at `src/tab.rs:197` and `src/tab.rs:250`. The
      failure-formatting tests (src/tab.rs:164-181 per baseline cites)
      carry no plugin id and stay (scenarios: "argv shape", "Directory
      containing spaces", "Popup request accepted", "Second popup while
      one is open").
- [x] 1.3 Update `src/main.rs:3` (module doc comment) and `src/main.rs:85`
      (unbound-invocation error hint) from `open-in.open` to
      `alexeyco.open-in.open`. Do NOT touch the stderr prefix `open-in: `
      at src/main.rs:52 and :127 — stable contract. Dispatch code needs no
      edit: src/main.rs checks only the presence of
      `HERDR_PLUGIN_ACTION_ID`, and the `mode_from` tests already pass the
      bare `"open"` (src/main.rs:228-229, :234, :244-245), which is what
      the corrected scenario pins (scenarios: "Entrypoint present
      alongside action id", "Action invocation opens the picker popup",
      "Run directly outside herdr").
- [x] 1.4 Expected result after this group alone: `cargo test --locked`
      and `cargo clippy --all-targets -- -D warnings` pass (only string
      literals changed); `grep -rn '"open-in"' src/` returns nothing;
      versions are still 0.3.0 — the manifest version is metadata with no
      compile-time coupling, so group 2 may follow in a later wave.

## 2. Version bump to 0.4.0

- [x] 2.1 Set `version = "0.4.0"` in `Cargo.toml:3` (package name
      `herdr-open-in` unchanged).
- [x] 2.2 Set `version = "0.4.0"` in `herdr-plugin.toml:4` (second edit
      of this file after 1.1; serialize the groups or expect a conflict)
      (scenarios: "Manifest fields are readable by herdr", "Version stays
      in sync with the crate").
- [x] 2.3 Refresh the lockfile: run `cargo update --workspace` (or any
      `cargo build`/`cargo test`) and verify
      `grep -A1 'name = "herdr-open-in"' Cargo.lock` shows
      `version = "0.4.0"` — docs/development.md:28-36 requires all three
      files to match in the same commit (scenario: "Version stays in sync
      with the crate").
- [x] 2.4 Expected result after this group alone: the build
      `cargo build --release --locked` succeeds (lockfile in sync), and
      the release tag check (`v0.4.0` vs Cargo.toml/herdr-plugin.toml)
      would pass.

## 3. CHANGELOG 0.4.0 entry

- [x] 3.1 Add a `## 0.4.0` section directly above `## 0.3.0` in
      `CHANGELOG.md`, Keep a Changelog format, with a `### Changed` block
      stating: the plugin id is renamed from `open-in` to
      `alexeyco.open-in` (author-namespaced), breaking existing
      keybindings — the qualified action id is now `alexeyco.open-in.open`;
      the upgrade steps: `herdr plugin uninstall open-in`,
      `herdr plugin install alexeyco/herdr-open-in`, rebind the key; old
      config/state under `open-in` is not migrated, so the last-chosen
      editor starts empty. End the section with the bullet
      "Version bumped to 0.4.0." in the file's existing style. Historical
      entries mentioning `open-in`/`open-in.open` (CHANGELOG.md:26, :33
      and older) stay verbatim — never rewrite history.

## 4. Documentation

- [x] 4.1 `README.md`: keybinding example `:36` →
      `command = "alexeyco.open-in.open"`; invoke commands `:43` and `:73`
      → `herdr plugin action invoke alexeyco.open-in.open`; `:74` →
      `herdr plugin log list --plugin alexeyco.open-in`. The title
      `# open-in` (`:1`) and banner alt (`:3`) are the project's
      colloquial name — unchanged.
- [x] 4.2 Add an `### Upgrading from 0.3.x` subsection at the end of the
      `## Install` section of `README.md` (immediately before `## Usage`):
      run `herdr plugin uninstall open-in`, then
      `herdr plugin install alexeyco/herdr-open-in`; rebind the key from
      `open-in.open` to `alexeyco.open-in.open`; config/state under the
      old id is not migrated — the last-chosen editor starts empty.
- [x] 4.3 `docs/how-it-works.md`: `:11` — change only the
      ``bind `open-in.open` `` hint to `alexeyco.open-in.open`; the
      `open-in:` stderr prefix in the same line stays. `:102` and `:127`
      → `--plugin alexeyco.open-in`; `:140` →
      `herdr plugin action invoke alexeyco.open-in.open`; `:144` →
      `herdr plugin log list --plugin alexeyco.open-in` and "the
      `alexeyco.open-in.open` invocation's". Line 3 ("`open-in` is a
      small Rust program") is prose — unchanged.
- [x] 4.4 `docs/development.md`: `:10` →
      `herdr plugin action list --plugin alexeyco.open-in`; `:11` →
      invoke `alexeyco.open-in.open`; `:12` →
      `log list --plugin alexeyco.open-in`; `:15` → the two
      `open-in.open` mentions and
      `herdr plugin unlink alexeyco.open-in` (the stale-link sentence
      already covers unlinking an old-id link).
- [x] 4.5 `AGENTS.md`: `:9` → invoke `alexeyco.open-in.open` and
      `log list --plugin alexeyco.open-in`; `:15` → reword to "The
      qualified action id is `alexeyco.open-in.open`" (the local action
      id remains `open`; keep the rest of the line unchanged, including
      the sentence that neither HERDR_PLUGIN_ENTRYPOINT_ID nor
      HERDR_PLUGIN_ACTION_ID is set).
- [x] 4.6 `.github/ISSUE_TEMPLATE/bug_report.yml:17` →
      `herdr plugin log list --plugin alexeyco.open-in`.
- [x] 4.7 Sweep: `grep -rn "open-in" README.md docs/ AGENTS.md .github/ CONTRIBUTING.md Makefile` — every remaining hit must be
      one of: the `herdr-open-in` crate/binary/repo name, an `OPEN_IN_*`
      env name, the `open-in:` stderr prefix, the colloquial title/prose
      (README.md:1, :3; docs/how-it-works.md:3), or an intentional old-id
      mention in the upgrade instructions (4.2) and CHANGELOG (3.1).
      Expected result after this group alone: `make fmt-check`
      (prettier --check "**/*.md") passes.

## 5. Baseline spec sync

- [x] 5.1 At sync/archive time the MODIFIED deltas merge into
      `openspec/specs/plugin-registration/spec.md` and
      `openspec/specs/herdr-pane-control/spec.md` by requirement header;
      Purpose paragraphs do NOT merge. Hand-edit the Purpose of
      `openspec/specs/plugin-registration/spec.md`: `:5` "the open-in
      plugin" → "the `alexeyco.open-in` plugin" and `:9` "action
      `open-in.open`" → "action `alexeyco.open-in.open`". The `open-in: `
      prefix sentence in the same Purpose (`:14`) stays — out of scope.
      `openspec/specs/herdr-pane-control/spec.md` Purpose names no plugin
      id — no edit.
- [x] 5.2 Post-sync sweep: `grep -rn "open-in" openspec/specs/` — every
      remaining hit must be the `herdr-open-in` binary path, an `OPEN_IN_*`
      name, the `open-in:` prefix, or the colloquial "the open-in plugin"
      prose in `openspec/specs/editor-catalog/spec.md:5` (project name,
      out of scope). The merged requirements must show id
      `alexeyco.open-in`, version `0.4.0`, `--plugin alexeyco.open-in`,
      and `HERDR_PLUGIN_ACTION_ID=open` in the dispatch scenario.

## 6. Verification

- [x] 6.1 `make check` green: `cargo fmt --check`,
      `cargo clippy --all-targets -- -D warnings`,
      `prettier --check "**/*.md"`, `cargo test --locked`,
      `cargo build --release --locked` (scenarios: "Entrypoint present
      alongside action id", "Directory containing spaces").
- [x] 6.2 `openspec validate namespace-plugin-id` reports VALID (certifies
      delta format only, not archive-ability).
- [x] 6.3 Manual dev loop on herdr 0.9.5: if an old-id link exists,
      `herdr plugin unlink open-in` once (local herdr state only — no repo
      file); `make build`; `herdr plugin link .`;
      `herdr plugin action invoke alexeyco.open-in.open` opens the picker
      popup; `herdr plugin log list --plugin alexeyco.open-in` lists the
      invocation record. Expect no `✓` last-choice mark on first open —
      the state dir under the new id starts empty (accepted, no
      migration). The user's local `~/.config/herdr/config.toml` keybinding
      is not touched (scenarios: "Manifest fields are readable by herdr",
      "Action is available in workspace context", "Action invocation opens
      the picker popup", "Popup request accepted").
- [x] 6.4 Error-text spot checks without herdr state: run
      `./target/release/herdr-open-in` with neither env var — exits 1 with
      stderr starts with `open-in:` and says neither
      HERDR_PLUGIN_ENTRYPOINT_ID nor HERDR_PLUGIN_ACTION_ID is set and
      to bind `alexeyco.open-in.open`; run
      `HERDR_PLUGIN_ENTRYPOINT_ID=bogus ./target/release/herdr-open-in` —
      exits 1 with the unknown-entrypoint error (scenarios: "Run directly
      outside herdr", "Bogus entrypoint string").
