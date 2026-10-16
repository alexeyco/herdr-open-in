# Tasks — restyle-picker-herdr-theme

## 1. Theme palette module

- [x] 1.1 Add `toml = "0.8"` (or current stable) as a runtime dependency in `Cargo.toml`.
- [x] 1.2 Create `src/theme.rs`: `TokenColor` (`Rgb`/`Ansi`/`Default`) with `fg_sgr()`/`bg_sgr()` helpers, and `Palette { accent, panel_bg, text, overlay1 }` (spec: "Theme palette resolution").
- [x] 1.3 Locate the real built-in palette table in the herdr 0.9.5 sources (the `src/app/state.rs` guess is UNVERIFIED), verify all 18 theme names and their hex values for `accent`/`panel_bg`/`text`/`overlay1` against the actual file, then embed the palettes and cite the source file and version in a comment and a `THEME_SOURCE_VERSION` constant.
- [x] 1.4 Implement `resolve_palette()`: `$HERDR_CONFIG_PATH` else `~/.config/herdr/config.toml`, parse `[theme] name` with `toml`; `terminal`/missing/unreadable/invalid/unknown → ANSI-16 fallback (accent=ANSI blue, overlay1=ANSI bright black, text/panel_bg=Default); resolve once at picker start.
- [x] 1.5 Unit tests for the resolver: built-in name, `HERDR_CONFIG_PATH` override, missing file, invalid TOML, unknown name, `name = "terminal"` (scenarios: "Built-in theme honored", "HERDR_CONFIG_PATH override", "Missing or broken config falls back", "Terminal or unknown name falls back").

## 2. Frame renderer rework

- [x] 2.1 Refactor `frame()` in `src/picker.rs` into a pure function taking `(list, selected, last_used_id, palette, width, scroll, visible)` and returning the whole frame string; keep hide-cursor/home/clear-tail prefix and blank top line.
- [x] 2.2 Render rows `" ▸ name ✓"`-style: `▸` marker only on the selected row, ` ✓` suffix on the row whose id equals the preselect id and never elsewhere (scenarios: "Check mark on the last-used row", "No check mark without a match").
- [x] 2.3 Style the selected row bold with fg=`panel_bg` on bg=`accent` via explicit SGR; unselected rows fg=`text` on bg=`panel_bg`; pad every row with spaces to the full drawable width inside the styling; never emit `\x1b[7m` (scenarios: "Multi-row frame", "Names padded to the widest", "No reverse video").
- [x] 2.4 Remove the old cyan-arrow/bold-only rendering, byte-length padding via `str::len()`, and any code path that omits the background paint.
- [x] 2.5 Draw the help footer as the frame's last line: `↑↓ select` / `enter open` / `esc cancel` text, fg=`overlay1` on bg=`panel_bg`, padded to full width, identical in every frame (scenarios: "Footer present and styled", "Footer is static").
- [x] 2.6 Wire `run()` to resolve the palette once before the first frame and pass `last_used_id` from the existing preselect value; keys, raw mode, outcome contract untouched.

## 3. Terminal size and scroll window

- [x] 3.1 Add `terminal_size()`: run `stty size` as a child process (same invocation pattern as the existing raw-mode stty calls), parse `"<rows> <cols>"`, fall back to 80×24 on any error (scenario: "Size read failure uses defaults").
- [x] 3.2 Compute `visible = drawable_rows - 2` (blank top line + footer, minimum 1) and `scroll = selected.saturating_sub(visible - 1)` before each frame; pass both into `frame()`; draw only rows `[scroll, scroll+visible)` (scenarios: "List fits — static frame", "Minimal bottom-scroll on downward selection", "Scrolling back up").
- [x] 3.3 Ensure no scrollbars, percentages, or row numbers appear in any frame; re-read the size every frame so a resize applies on the next redraw.

## 4. Manifest height

- [x] 4.1 Raise `[[panes]] picker` `height` from 14 to 16 in `herdr-plugin.toml:17`; keep `width = "40%"` (plugin-registration delta: "Picker popup geometry"; Editor pane placement scenario unchanged — no work).
- [x] 4.2 Sanity-check inner height: 16 − 2 border rows ≥ 11 catalog rows + blank line + footer (scenario: "Full catalog fits without scrolling").

## 5. Tests

- [x] 5.1 Rewrite the existing frame-format tests in `src/picker.rs` (old cyan/bold-only expectations) against the new pure `frame()` with an injected fixture palette.
- [x] 5.2 Add ANSI-snapshot tests pinning exact SGR byte sequences for: selected row (catppuccin fixture: bold + `38;2;24;24;37` + `48;2;137;180;250`, full-width padding), unselected row, footer line, ` ✓` suffix placement, and frame boundaries (clear sequences, blank top line, footer last).
- [x] 5.3 Add snapshot tests for the scroll window: N=8 with visible=6 — selection at index 6 renders rows `1..=6` (window scrolled by one, selected at the window bottom); selection back to 0 renders rows `0..=5`; selection at the last index 7 renders rows `2..=7`.
- [x] 5.4 Keep the invariant tests: exactly one `▸` per frame and no `\x1b[7m` anywhere (now spec-pinned under "Frame rendering").
- [x] 5.5 Add a fallback-palette snapshot test: selected row uses ANSI blue background with no fg/bg sequence for panel_bg/text; footer uses `90` fg (scenario: "Fallback palette rendering").
- [x] 5.6 Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; fix failures.

## 6. Documentation

- [x] 6.1 Update `README.md:42-56` picker section: themed one-column list, accent highlight, `✓` last-used mark, help footer, scroll behavior, theme source (`[theme] name`, built-ins only, ANSI-16 fallback) and limits (no `[theme.custom]`, no auto_switch).
- [x] 6.2 Update `docs/how-it-works.md:109-121` rendering description to the new frame format and palette resolution.
- [x] 6.3 Correct the dependency claim ("single runtime dependency: `serde_json`") in `README.md`/`AGENTS.md` to `serde_json` + `toml`.
- [x] 6.4 At sync/archive time, update the Purpose paragraph of `openspec/specs/editor-picker/spec.md`: its "There is no header, help text, or search/filter UI" line will contradict the new "Help footer" requirement (deltas cannot modify Purpose).

## 7. Acceptance eyeball check

- [x] 7.1 Build release (`cargo build --release --locked`), `herdr plugin link .`, invoke `open-in.open`; compare the popup side by side with herdr's settings theme dialog on the same theme: same accent highlight, muted footer tone, `✓` idiom, one column, full-width selection, no scroll for the full catalog at height 16.
- [x] 7.2 Repeat with `HERDR_CONFIG_PATH` pointing to a config with `name = "terminal"` and to a nonexistent file: ANSI-16 fallback looks sane in a plain terminal theme.

## 8. Bottom-pinned footer

- [x] 8.1 `frame()` takes `height` and emits exactly `height` lines: blank top line, scroll-window rows, blank filler lines, one blank spacer, footer last; blank lines (top/filler/spacer) styled bg=`panel_bg` only (no fg), padded to full width; under the ANSI-16 fallback blank lines emit no SGR at all (scenarios: "Footer pinned to the bottom", "Blanks painted as panel").
- [x] 8.2 Pass the freshly-read drawable rows into `frame()` each frame from `run_with`; `visible_rows` becomes `rows - 3` (minimum 1); scroll formula `scroll = selected.saturating_sub(visible - 1)` unchanged (supersedes task 3.2's `rows - 2`).
- [x] 8.3 Update the ANSI snapshot tests to the new geometry: byte-exact tests for scenarios "Footer pinned to the bottom" and "Blanks painted as panel", and revised budget math (`rows - 3`, filler/spacer lines) in the existing frame and scroll-window tests; re-confirm the ticked 4.2 arithmetic against the revised spacer-inclusive geometry (14 = 1 + 11 + 1 + 1).
- [x] 8.4 Update `README.md` and `docs/how-it-works.md` wording: the help footer is docked to the popup bottom under one mirror blank spacer; at height 16 the 11-entry catalog fills the list area exactly.
- [x] 8.5 Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; fix failures.

## 9. Alphabetical catalog order

- [x] 9.1 Reorder the `CATALOG` array in `src/editors.rs` by display name (case-insensitive) to: Android Studio, Cursor, helix, IntelliJ IDEA, nvim, PyCharm, vim, VS Code, WebStorm, Windsurf, Zed; ids and all per-entry fields unchanged (spec: editor-catalog delta "Catalog entries are ordered alphabetically").
- [x] 9.2 Update the pinned test `catalog_lists_eleven_rows_in_menu_order` expectations; add or extend a test asserting the names are in case-insensitive alphabetical order; fix any order-dependent snapshot fixtures/tests broken by the reorder (e.g. `eight_items()` in `src/picker.rs` takes the first 8 entries of `CATALOG`, and the scroll snapshots enumerate names explicitly).
- [x] 9.3 Adjust the README picker-order sentence (one row per editor, alphabetical) and `docs/how-it-works.md` if it mentions catalog order.
- [x] 9.4 Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; fix failures.
- [x] 9.5 At sync/archive time, update the Purpose paragraph of `openspec/specs/editor-catalog/spec.md`: its line that calls menu render order "an implementation detail" becomes a contract under the new ADDED ordering requirement (deltas cannot modify Purpose).

## 10. No trailing line break (full-height frame fix)

- [x] 10.1 `frame()` emits exactly `height - 1` line breaks: every line except the footer is newline-terminated, the footer line is not — the frame string ends on the footer's last byte (spec: "Frame rendering", trailing-line-break clause).
- [x] 10.2 Update frame-shape assertions: `assert_frame_shape`/`frame_lines` expect a `\r\n` count of `height - 1`, byte-exact snapshots drop the final `\r\n`, and add a snapshot assertion pinning that the frame ends on the footer's last byte (scenario: "Footer pinned to the bottom").
- [x] 10.3 Docs: check `README.md` and `docs/how-it-works.md` for any sentence describing frame boundaries; adjust only if inaccurate (verify — probably none).
- [x] 10.4 Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`; fix failures.
