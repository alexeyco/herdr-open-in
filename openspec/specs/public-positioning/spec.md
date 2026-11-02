# public-positioning Specification

## Purpose

The README `## Comparison` section presents a factual, sourced comparison
of this plugin and five other editor-launch tools for herdr. Contributors
manually review the claims; a monthly workflow checks links and release
data.

## Requirements

### Requirement: README carries a sourced Comparison section

The README SHALL place `## Comparison` after `## Usage` and before
`## Editor catalog`. The section SHALL contain an introductory sentence,
a legend, a seven-row matrix, its source footnotes, an audit date in the
form `All cells checked YYYY-MM-DD against the linked public sources.`,
a detailed comparison table, a factual positioning paragraph, and links
to other herdr plugins, in that order.

The matrix SHALL compare this plugin with timofey-TK/herdr-open-in-editor,
harish-zuper/herdr-open-project-with, agentience/herdr-plugin-ide-jump,
alex-devdone/herdr-cursor-open, and upnorthmedia/herdr-open-in-editor.
Its criteria SHALL be, in order: opens the workspace via a herdr action;
picker of choices at one keypress; editors discovered, not preconfigured;
terminal editors run inside herdr; JetBrains-family coverage; remembers
the last choice; runs on both Linux and macOS.

The detailed table SHALL cover editor selection, external runtime
dependencies, platforms, latest release, and license/language. The
"also exists" list SHALL link Andreslvc/herdr_plugin,
gadenbuie/herdr-positron, kongdd/herdr-open, y-hirakaw/herdr-launcher-pane,
and T0mSIlver/hither.

#### Scenario: Section structure and matrix

- **WHEN** a reader opens the README Comparison section
- **THEN** it appears between Usage and Editor catalog, follows the
  required order, and has seven criteria across six columns

### Requirement: Comparison claims are sourced and neutral

Every factual cell in both tables SHALL be supported by a uniquely
numbered footnote linking to a public source, a direct link in the
detailed table, or the legend's not-found rule for a bare `❌`. Footnotes
SHALL be numbered in reading order. Every `⚠️`, competitor `✅`, and
positively documented absence SHALL have a footnote. The legend SHALL
define `✅`, `⚠️`, and `❌`.

Latest-release cells SHALL link to the published tag or to the
repository's releases page when no release exists. License/language
cells SHALL link to the repository. The audit date SHALL record when
both tables were manually checked.

The section SHALL use neutral, factual language. Its positioning
paragraph SHALL limit any uniqueness claim to the combination of a
picker for locally discovered editors, terminal editors running in
herdr tabs, macOS app-bundle fallback for GUI editors without a CLI, and
last-choice memory. It SHALL NOT claim broader or single-feature
uniqueness.

#### Scenario: Cells have evidence and the claims stay neutral

- **WHEN** a factual cell or the positioning paragraph is reviewed
- **THEN** the cell has an allowed source, absences use the legend rule,
  and the paragraph claims only the supported four-part combination

#### Scenario: Release and license cells link to their sources

- **WHEN** the detailed table's release and license/language rows are
  inspected
- **THEN** release cells link to a tag or the releases page, and
  license/language cells link to the corresponding repository

### Requirement: Contributors manually re-audit comparison claims

Under `## Conventions`, CONTRIBUTING.md SHALL require an edit to the
README Comparison section to re-audit every cell in both tables against
the linked public sources and update the audit date in the same change.
This review covers the factual meaning of cells; the scheduled checker
does not perform that semantic review or change the date.

#### Scenario: Comparison edits receive a manual audit

- **WHEN** a contributor changes comparison content
- **THEN** every cell is checked against its source and the audit date
  is updated in the same change, regardless of the scheduled check

### Requirement: Monthly comparison freshness is checked automatically

The repository SHALL provide `.github/workflows/comparison-freshness.yml`
with workflow display name `Comparison Freshness`, job key
`comparison-freshness`, and job display name `Comparison Freshness`. It
SHALL run at `0 6 1 * *` and support `workflow_dispatch`.

The checker SHALL compare configured upstream latest-release tags with
the README release cells, check HTTP(S) links in the Comparison section,
and confirm configured release cells are present. It SHALL NOT infer or
change the factual meaning of comparison claims or update the manual
audit date. Release-tag drift SHALL produce a reviewable rolling pull
request; other reachable drift SHALL be reported in one issue; an
unavailable source SHALL fail the run without creating either artifact.

#### Scenario: Scheduled and manual checks are available

- **WHEN** the first day of a month arrives at 06:00 UTC or a maintainer
  dispatches the workflow
- **THEN** the named job checks the README Comparison section

#### Scenario: Drift is routed safely

- **WHEN** release tags drift, another checked source drifts, or a
  required source is unavailable
- **THEN** release drift opens or updates a pull request, other drift
  opens or updates an issue, and unavailable sources fail without
  creating either artifact or writing directly to the base branch

### Requirement: Other herdr plugins list links current sibling plugins

The README's "Other herdr plugins" list SHALL contain exactly herdr-gitui
(`https://github.com/alexeyco/herdr-gitui`) and herdr-review
(`https://github.com/alexeyco/herdr-review`), each with a one-line
description, and SHALL NOT list herdr-revdiff.

#### Scenario: Cross-link list contents

- **WHEN** the "Other herdr plugins" list under the README intro is read
- **THEN** it names herdr-gitui and herdr-review with GitHub links and
  one-line descriptions, and has no herdr-revdiff entry
