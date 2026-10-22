# Development

Development happens against a locally linked plugin; no GitHub install needed.

## Dev loop

```
herdr plugin link .              # register the working directory as a plugin
make build                       # produces ./target/release/herdr-open-in
herdr plugin action list --plugin alexeyco.open-in
herdr plugin action invoke alexeyco.open-in.open
herdr plugin log list --plugin alexeyco.open-in
```

An earlier, pre-rename plugin link may still point at a directory that no longer exists; unlink that stale plugin first, then link again. `plugin link` does not run build commands, so rebuild after every source change. The picker dialog needs a real terminal, so drive it through the `alexeyco.open-in.open` action (a popup) rather than running `./target/release/herdr-open-in` directly. That popup runs as its own pane process, so herdr does not capture its stderr or exit status: `herdr plugin log list --plugin alexeyco.open-in` lists the plugin's action records (the `alexeyco.open-in.open` invocation's own exit status, stdout and stderr), not the popup pane it opens. Errors the picker raises before the dialog starts — a missing context, a bad `workspace_id`, or no detected editor — can be read directly by running the binary from a terminal with `HERDR_PLUGIN_ENTRYPOINT_ID=picker` and a synthetic `HERDR_PLUGIN_CONTEXT_JSON`. Unlink with `herdr plugin unlink alexeyco.open-in`.

## Code structure

`src/features/` contains editor selection and execution flows. `src/adapters/` handles environment values, herdr context and pane requests, terminal UI, editor discovery and processes, and saved choice state. Keep decisions in features and system calls in adapters; feature tests use `mockall` mocks for their host ports.

## Checks

```
make lint        # cargo fmt --check + cargo clippy
make test        # cargo test
make build       # release build
make fmt-check   # prettier check
```

`make check` runs all of them. CI runs the same commands verbatim (it does not use make); see [.github/workflows/ci.yml](../.github/workflows/ci.yml).

## Version sync

The version lives in three files that must match:

- `Cargo.toml`
- `Cargo.lock`
- `herdr-plugin.toml`

Bump all three in the same commit. A release is cut by pushing a tag `v<version>` (e.g. `v0.2.0`); the release workflow fails if the tag does not match `Cargo.toml` and `herdr-plugin.toml`. For release steps, see [CONTRIBUTING.md](../CONTRIBUTING.md).

## Notes

- herdr validates `min_herdr_version` against the running herdr binary; linking with an older herdr is refused.
- Usage and requirements are documented in the [README](../README.md); behavior details in [how-it-works.md](how-it-works.md).
