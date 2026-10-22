# Design: Simplify the editor picker

## Approach

Use ratatui 0.30.2 with default features disabled and its `crossterm` feature enabled, matching the dependency configuration and picker pattern in `../herdr-review`. Do not add a standalone `crossterm` dependency. Keep the editor picker as the UI boundary: a pure selection/update layer accepts events and owns the selected index, while a renderer receives the editor list and selection and draws the compact borderless list. The terminal event loop connects these pieces and delegates terminal lifecycle, sizing, clipping, and scrolling to ratatui/crossterm. CI compiles and tests the implementation on all current targets, including Windows; runtime plugin support remains Linux and macOS as declared in `herdr-plugin.toml`.

The caller handles the one-editor case before opening the picker. For multiple editors, preserve last-choice preselection and the current selection/cancel, best-effort persistence, and launch flow. Put those decisions in `features` and isolate environment, filesystem, terminal, editor-process, and herdr calls in `adapters`; feature tests mock host traits with `mockall`. Use `anyhow` for application flow and process errors, adding context at integration boundaries. Remove theme lookup and palette data. The displayed list uses terminal-default colors, basic selected-row styling, and no footer or last-used marker.

## Files to touch

- `Cargo.toml` — add ratatui 0.30.2 with default features disabled and its `crossterm` feature enabled, plus anyhow at runtime and mockall for tests; remove dependencies used only by the replaced theme handling if no longer used elsewhere.
- `Cargo.lock` — record the dependency graph update.
- `src/main.rs` — keep only environment-based mode dispatch.
- `src/features/` — own the editor-selection and terminal-editor execution flows behind small host traits, with mockall-based tests.
- `src/adapters/` — group herdr context, editor discovery and launch, terminal picker, persisted choice, and child-process integrations.
- `src/theme.rs` — remove the obsolete theme-resolution module.

## Trade-offs

The dialog inherits terminal-default colors so it remains legible under user terminal themes without project-maintained palette logic. Its popup is 38 columns wide, with a `›` marker on the selected row and one blank row above and below the list. Ratatui/crossterm own terminal protocol behavior, reducing the amount of terminal-specific code the project must maintain. The test backend validates rendered cells without asserting terminal escape byte sequences.
