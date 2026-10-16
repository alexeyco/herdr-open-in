# plugin-registration — delta for restyle-picker-herdr-theme

## MODIFIED Requirements

### Requirement: Manifest declares picker and editor panes

The manifest SHALL declare two panes, both running
`./target/release/herdr-open-in`: a `picker` pane titled "Open in" placed as
a popup with width `40%` and height `16`, and an `editor` pane titled
"Editor" placed as a tab. The height is part of this contract: the popup's
inner area (height minus the two border rows herdr draws) must fit the full
11-entry catalog plus the frame chrome without scrolling.
(herdr-plugin.toml:12-24)

At height 16 the inner 14 lines are exactly 1 blank top line + 11 catalog
rows + 1 blank spacer + 1 help footer — the catalog fits at exact capacity
with no spare row; further catalog growth triggers the picker's scroll
window by design.

#### Scenario: Picker popup geometry

- **WHEN** herdr opens the `picker` pane
- **THEN** it renders a popup titled "Open in" sized 40% wide and 16 lines
  tall

#### Scenario: Full catalog fits without scrolling

- **WHEN** the popup opens on a machine with all catalog entries detected
- **THEN** all 11 editor rows, the blank top line, the blank spacer line, and
  the help footer occupy all 14 inner lines in the first frame with the scroll
  window at zero and no row hidden

#### Scenario: Editor pane placement

- **WHEN** herdr opens the `editor` pane
- **THEN** it is placed as a tab titled "Editor"
