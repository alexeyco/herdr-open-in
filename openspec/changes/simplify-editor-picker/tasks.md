# Tasks: Simplify the editor picker

## 1. Replace terminal and rendering implementation

- [x] 1.1 Add ratatui 0.30.2 with default features disabled and its `crossterm` feature enabled; do not add standalone `crossterm` unless implementation evidence requires it. Update `Cargo.lock`; remove dependencies used only by the replaced theme resolver. Confirm dependency usage with `rg` and ensure the crate builds after this group. (scenario: "Build and test on every CI target")
- [x] 1.2 Replace `src/adapters/picker.rs` custom ANSI rendering, `stty` raw mode, terminal sizing, scrolling, and escape parsing with a ratatui/crossterm event loop, compact terminal-default list renderer, and independent selection/update logic; use the test backend to verify list order and selected-row styling. Ensure terminal restoration on selection, cancellation, and errors. (scenarios: "Render choices in a test backend", "Render without herdr theme configuration", "Move down and select", "Movement stays within list bounds", "Escape cancels without launching", "Unsupported keys have no effect", "Keep selection visible in long lists", "Restore the terminal on exit")
- [x] 1.3 Remove `src/theme.rs` and its module declaration from `src/main.rs`; grep all theme imports and references and ensure the crate builds after removal. (scenario: "Render without herdr theme configuration")

## 2. Integrate dialog behavior and preserve outcomes

- [x] 2.1 Update `src/main.rs` to launch the sole detected editor directly and open the picker only for multiple editors; preserve multi-editor last-choice preselection, best-effort state saving, cancellation, and selected-editor launch. Test the one-editor and multi-editor routing. (scenarios: "Launch a single available editor directly", "Open the dialog for multiple editors", "Preselect the matching last choice", "Default to the first editor", "Save and launch the selected editor")
- [x] 2.2 Preserve empty-list and terminal-error propagation through the existing caller contract; verify empty input produces no frame and setup, input, or rendering errors reach the caller with nonzero exit status. (scenarios: "Reject an empty list before rendering", "Report terminal errors to the caller")
- [x] 2.3 Update the usage and development documentation for direct launch with one editor, dialog controls, terminal-default styling, and the ratatui/crossterm implementation.
- [x] 2.4 Separate editor-selection and terminal-editor flows into `features`, group system boundaries in `adapters`, and test the flows through host ports.
- [x] 2.5 Use anyhow for application and process errors, with context at integration boundaries; use mockall to generate feature-port mocks and assert interaction order and outcomes.

## 3. Verify platform coverage and project checks

- [ ] 3.1 Compile and run the picker tests on every current CI target, including Windows, while preserving `herdr-plugin.toml` runtime support for Linux and macOS only. (scenarios: "Build and test on every CI target", "Preserve runtime plugin platforms")
- [x] 3.2 Replace ANSI/stty implementation-coupled tests with selection and test-backend rendering tests, then run the checks in `docs/development.md`. Confirm all scenarios in this change pass and `make check` succeeds.

> DEFERRED: 3.1 — Windows test execution needs the existing Windows CI runner because this host has no MSVC linker — run the Windows CI job after pushing the reviewed branch.
