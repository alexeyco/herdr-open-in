# editor-catalog — delta for restyle-picker-herdr-theme

## ADDED Requirements

### Requirement: Catalog entries are ordered alphabetically

The catalog SHALL list its editor entries ordered by display name, compared
case-insensitively (ASCII fold, no locale collation). Only the order of the
source list changes: ids, kinds, commands, and every other per-entry field
stay as they are. The picker's display order follows from the existing
"Catalog order preserved" contract (editor-detection) and the menu's
catalog-order rule, so no other capability needs a delta.

#### Scenario: Alphabetical display order

- **WHEN** the picker receives all catalog entries
- **THEN** the rows appear in the order Android Studio, Cursor, helix,
  IntelliJ IDEA, nvim, PyCharm, vim, VS Code, WebStorm, Windsurf, Zed —
  the catalog's display names in case-insensitive alphabetical order (VS
  Code sits between vim and WebStorm)
