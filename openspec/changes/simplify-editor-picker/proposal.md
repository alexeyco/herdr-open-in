# Simplify the editor picker

## Why now

The picker in `src/picker.rs` owns raw terminal mode, ANSI rendering, sizing, scrolling, and escape-sequence parsing. `src/theme.rs` also reads herdr theme configuration solely to style the dialog. The reference picker in `../herdr-review` uses ratatui and crossterm for terminal behavior and supports rendering through a test backend.

## What changes

Replace the custom terminal implementation with a compact ratatui dialog using terminal-default colors. Launch the sole detected IDE immediately; show the dialog when multiple IDEs are available. Keep the dialog's supported controls to Up, Down, Enter, and Escape. Separate application flows from herdr, terminal, process, and persistence adapters so behavior can be tested through mockable host ports. Use anyhow for application errors and mockall for feature tests. Make rendering and selection independently testable, including rendering with a test backend, and compile and test the implementation on every CI target. Runtime plugin support remains Linux and macOS, as declared in the plugin manifest.

Example: one detected IDE launches directly; with two IDEs, the dialog lists both and Up/Down plus Enter selects one, while Escape cancels.

## Capabilities

- `editor-picker`: replace terminal presentation and input handling, and define single-choice behavior.
- Existing editor detection, editor launch, last-choice persistence, and herdr integration remain outside this change.

ASSUMPTION: Popup width and content layout follow `../herdr-review`'s compact borderless style: a 38-column width, a selected-row marker, and one blank row above and below the list. Use terminal-default colors and basic text styling.
