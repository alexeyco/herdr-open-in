# Proposal: Focus terminal editor tabs

## Why

When the plugin opens a terminal editor tab, herdr currently leaves focus on the agent tab. After herdr acknowledges the new pane, the plugin will focus the opened tab so the selected `nvim`, `vim`, or `helix` editor is ready for input. GUI editor launch behavior remains unchanged.

The user tested `--focus` on `plugin pane open` and `plugin pane focus <PANE_ID>` in a linked herdr workspace; neither changed the active tab. Local read-only help shows `herdr tab focus --help` → `Usage: herdr tab focus <tab_id>`. The `plugin pane open` response places `tab_id` at `result.plugin_pane.pane.tab_id`, alongside `pane_id`.

## What Changes

After parsing the pane id and tab id from the terminal-tab open response, the plugin will run `$HERDR_BIN_PATH tab focus <TAB_ID>`. The terminal-tab open argv keeps its baseline shape and does not include `--focus`; the editor process, workspace directory, pane placement, and GUI launch path keep their current behavior. The picker popup retains its own `--focus` option. This affects the `herdr-pane-control` capability.
