# public-positioning Specification

## Purpose

The public-positioning capability is the README `## Comparison` section:
what it contains, where it sits, how every factual cell is sourced, what
the plugin may claim about itself relative to the five competitor
columns, and the manual re-audit rule in CONTRIBUTING.md that keeps the
section from rotting. It exists so the positioning stays factual,
neutral, and auditable.

## Requirements

### Requirement: README carries a sourced, dated Comparison section

The README SHALL carry a `## Comparison` section placed directly after
the `## Usage` section and before `## Editor catalog` — the `##` heading
order is Requirements, Install, Usage, Comparison, Editor catalog,
Development, Changelog, License, with every other heading unchanged.
Inside the section, in this order:

1. one intro sentence stating that the section compares this plugin,
   cell by cell as documented in their public sources, with the other
   editor-launch tools that integrate with herdr, and the legend line
   `✅ documented in the linked sources · ⚠️ partial or implemented differently · ❌ not found in the public sources checked`;
2. the at-a-glance matrix: 7 criterion rows × 6 columns (this plugin
   plus the five competitors `timofey-TK/herdr-open-in-editor`,
   `harish-zuper/herdr-open-project-with`,
   `agentience/herdr-plugin-ide-jump`, `alex-devdone/herdr-cursor-open`,
   `upnorthmedia/herdr-open-in-editor`), each competitor column header
   linking its GitHub repo and displayed by short tool name (the
   `herdr-`/`herdr-plugin-` prefix dropped), except the two repos sharing
   the name `herdr-open-in-editor`, whose headers display the owner
   (`timofey-TK`, `upnorthmedia`) to disambiguate — while the subject
   column header displays `herdr-open-in`; every cell one of ✅/⚠️/❌;
3. the numbered footnote block whose definitions carry the public source
   links, markers unique and numbered in reading order;
4. the dated line
   `All cells checked 2026-10-18 against the linked public sources.`;
5. a `<details>` block summarized `Detailed comparison` holding the
   detailed table with the rows Editor selection, External runtime
   dependencies, Platforms, Latest release, and License / language over
   the same six columns;
6. the positioning paragraph;
7. an "also exists" lead-in line and a one-line bullet list naming the
   five further same-space tools with their repo links:
   `Andreslvc/herdr_plugin`, `gadenbuie/herdr-positron`,
   `kongdd/herdr-open`, `y-hirakaw/herdr-launcher-pane`,
   `T0mSIlver/hither`.

The seven matrix criteria SHALL be, in order: opens the workspace via a
herdr action; picker of choices at one keypress; editors discovered, not
preconfigured; terminal editors run inside herdr; JetBrains-family
coverage; remembers the last choice; runs on both Linux and macOS.

#### Scenario: Heading order puts Comparison after Usage

- **WHEN** the rendered README's `##` headings are read top to bottom
- **THEN** `## Comparison` appears after the `## Usage` section and
  before `## Development`, the full order is Requirements, Install,
  Usage, Comparison, Editor catalog, Development, Changelog, License, and
  no other heading moved

#### Scenario: Section anatomy in fixed order

- **WHEN** a reader opens the `## Comparison` section
- **THEN** its parts appear in the order: intro sentence + legend line,
  matrix, footnote block, dated `All cells checked 2026-10-18 …` line,
  `<details>` "Detailed comparison" table, positioning paragraph,
  also-exists list

#### Scenario: Seven criteria across six columns

- **WHEN** the matrix is read
- **THEN** it holds exactly the seven criterion rows named above over
  six columns — this plugin plus timofey-TK/herdr-open-in-editor,
  harish-zuper/herdr-open-project-with, agentience/herdr-plugin-ide-jump,
  alex-devdone/herdr-cursor-open, upnorthmedia/herdr-open-in-editor —
  each competitor header a link to its GitHub repo displayed by
  short tool name (owner for the two `herdr-open-in-editor` repos), and
  every cell one of ✅/⚠️/❌ (with a footnote marker
  where the requirement "Every factual cell is sourced and the legend
  defines the states" demands one)

#### Scenario: Also-exists list names the five further tools

- **WHEN** the reader reaches the end of the section
- **THEN** an also-exists list gives one line each, with a repo link,
  for Andreslvc/herdr_plugin, gadenbuie/herdr-positron, kongdd/herdr-open,
  y-hirakaw/herdr-launcher-pane, and T0mSIlver/hither

### Requirement: Every factual cell is sourced and the legend defines the states

Every factual cell of both tables SHALL carry a source that supports it:
a uniquely numbered footnote whose definition carries a public link (the
competitor's GitHub repo, or — for file-level evidence — a blob URL at
that repo), a direct external link inside the detailed cell, or — for
bare ❌ matrix cells — the legend's `not found in the public sources
checked` rule: a bare ❌ states that the feature is not documented in the
project's public sources, negative searches performed 2026-10-18, and is
a sourced negative claim under the dated line. Every ⚠️ cell, every
competitor ✅ cell, and every ❌ cell whose absence is positively
documented in the project's public sources (a stated platform list, a
stated "no picker" design) SHALL carry a footnote; a subject ✅ cell
whose evidence is self-evident from this repository's README needs none.
Markers SHALL be unique and numbered in reading order — matrix rows top
to bottom, cells left to right — none
reused; identical evidence used at two positions yields two numbered
definitions with identical text. In the detailed table's Latest release
row, each competitor cell SHALL carry the shape
`[<tag>](https://github.com/<owner>/<repo>/releases/tag/<tag>)` when the
repo publishes a release and `[no releases](https://github.com/<owner>/<repo>/releases)`
when it publishes none; the subject cell links
`https://github.com/alexeyco/herdr-open-in/releases` without a pinned
tag. Each License / language cell SHALL carry the shape
`[<license> · <language>](https://github.com/<owner>/<repo>)`, or
`no license file` in place of `<license>` when the repo publishes none.
The dated line SHALL sit directly under the footnote block and cover
both tables; it records the human audit performed 2026-10-18.

#### Scenario: Legend defines the three states

- **WHEN** a reader sees ✅, ⚠️, or ❌ cells in the matrix
- **THEN** the legend line directly above the matrix defines all three
  states, and the ❌ wording states the bare-❌ rule that keeps
  unfootnoted ❌ cells sourced negative claims — the search date
  (2026-10-18) is carried by the dated line under the footnote block,
  not by the legend text itself

#### Scenario: Questioned cell carries its source

- **WHEN** any factual cell — matrix or detailed — is questioned
- **THEN** it carries a public source: a uniquely numbered footnote whose
  definition links the competitor's GitHub repo or a blob URL at it, a
  direct external link in the detailed cell, or — for a bare ❌ matrix
  cell — the legend's not-found rule with the dated line; no cell rests
  on an uncited claim

#### Scenario: Dated line is verbatim

- **WHEN** the footnote block is read to its end
- **THEN** the next line is exactly
  `All cells checked 2026-10-18 against the linked public sources.`

#### Scenario: Release cells carry the linked shape

- **WHEN** the detailed table's Latest release row is inspected
- **THEN** each competitor cell is either
  `[<tag>](https://github.com/<owner>/<repo>/releases/tag/<tag>)` or
  `[no releases](https://github.com/<owner>/<repo>/releases)`, with no
  release date in the cell, and the subject cell links
  `https://github.com/alexeyco/herdr-open-in/releases` with no pinned tag

### Requirement: Comparison stays neutral and claims only the defensible combination

The section SHALL use neutral, factual language — no evaluative verdicts
("better", "worse", "inferior"); absences are stated as sourced facts
linked to the competitor's public sources or covered by the legend's
not-found rule. The positioning paragraph's uniqueness claim SHALL be
limited to the combination of (1) a picker of editors discovered from
the machine, (2) terminal editors (nvim, vim, helix) running in herdr
tabs, (3) the macOS app-bundle fallback for GUI editors whose CLI is
missing, and (4) last-choice memory — the combination no compared
competitor column matches in the matrix. No broader "only tool that …"
statement SHALL appear, and no single-criterion uniqueness SHALL be
claimed beyond what the matrix cells evidence.

#### Scenario: Unique-combination claim only

- **WHEN** the positioning paragraph asserts what distinguishes this
  plugin
- **THEN** the claim names exactly the four-part combination above, is
  phrased as a combination, and no other criterion is claimed as unique

#### Scenario: Absence stated neutrally

- **WHEN** the tables record what a competitor lacks — a ❌ or ⚠️ cell,
  such as a macOS-only platform list or a fixed configured editor
- **THEN** the absence is stated as a fact linked to the competitor's
  public source or covered by the legend's not-found rule, and no
  evaluative adjective appears anywhere in the section

### Requirement: CONTRIBUTING carries the manual re-audit rule

CONTRIBUTING.md SHALL carry, under `## Conventions`, one rule sentence:
an edit that touches the README `## Comparison` section re-audits every
cell of both tables against the linked public sources and updates the
`All cells checked …` date in the same change. This rule SHALL be the
only freshness mechanism the capability requires — no bot and no script
is required for, or may be introduced as part of, comparison freshness.

#### Scenario: Re-audit rule exists

- **WHEN** a contributor reads `## Conventions` in CONTRIBUTING.md
- **THEN** one bullet states that touching the README `## Comparison`
  section requires re-auditing the cells against the linked public
  sources and updating the `All cells checked …` date in the same change

#### Scenario: Freshness mechanism stays manual

- **WHEN** the repository is inspected for comparison-freshness
  enforcement
- **THEN** the CONTRIBUTING rule is the whole mechanism: no bot, no
  scheduled workflow, and no script guards the Comparison section

### Requirement: Other herdr plugins list links the current sibling plugins

The README's "Other herdr plugins" list SHALL hold exactly two entries —
herdr-gitui (`https://github.com/alexeyco/herdr-gitui`) and herdr-review
(`https://github.com/alexeyco/herdr-review`), each with a one-line
description — and SHALL NOT list herdr-revdiff, mirroring herdr-review's
own list, which already links this plugin.

#### Scenario: Cross-link list contents

- **WHEN** the "Other herdr plugins" list under the README intro is read
- **THEN** it names herdr-gitui and herdr-review with working GitHub
  links and one-line descriptions, and no herdr-revdiff entry remains
