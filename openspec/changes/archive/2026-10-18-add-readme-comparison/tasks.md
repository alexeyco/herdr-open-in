# Tasks — add-readme-comparison

All groups are docs-only: after any single group, `cargo build`,
`cargo test`, and `make lint` are unaffected — no `src/`, `Cargo.*`, or
`herdr-plugin.toml` file is touched. Groups 1–4 are implemented as
ticked; they ran against the pre-ruling artifacts (an 8-criteria ×
6-column matrix, 27 footnote definitions), so their task text keeps the
counts as executed while their scenario citations name the revised
headings. Group 5 applies the post-review ruling recorded in
proposal.md: the Remote/SSH row leaves the matrix, giving the
7-criteria × 6-column matrix with the 24 footnote definitions of
revised design.md §3–§4; the revised delta spec holds 13 scenarios and
every one is cited below. `make fmt-check` was green after group 4's
`make fmt`, may go red while group 5's edits are unpadded, and is
expected green again after task 5.4 re-runs `make fmt`; write tables
unpadded and let prettier pad them (design.md §12). Scenario names
cited below match `#### Scenario:` headings in
`openspec/changes/add-readme-comparison/specs/public-positioning/spec.md`
verbatim.

## 1. README Comparison section (README.md)

- [x] 1.1 Replace the "Other herdr plugins" list (README.md:9-12) with
      the exact two-entry alphabetical list from design.md §9 —
      herdr-gitui kept, herdr-review added, herdr-revdiff deleted.
      Covers scenario "Cross-link list contents".
- [x] 1.2 Fetch, read-only, the five competitor repos named in
      design.md §2: current release tag (or absence of releases) from
      each releases page, license name (or absence of a license file),
      and primary language. Record the ten cell values for the detailed
      table's Latest release and License / language rows per the shapes
      in design.md §6. Feeds scenario "Release cells carry the linked
      shape"; do not invent a value the fetch did not return.
- [x] 1.3 Insert `## Comparison` between `## Usage` and `## Development`
      with the intro sentence and legend line from design.md §5 and the
      full 8×6 matrix from design.md §3, cell for cell including the
      bare-❌ cells and the subject's remote/SSH ❌. Covers scenarios
      "Heading order puts Comparison after Usage", "Seven criteria
      across six columns", and "Legend defines the three states".
- [x] 1.4 Write the footnote block: the 27 definitions from design.md §4
      with their exact texts, source links, and reading-order numbers,
      and place the `f<n>` markers in the matrix cells exactly as the
      §3 map shows. Covers scenario "Questioned cell carries its
      source".
- [x] 1.5 Add the dated line directly under the footnote block, exactly
      as design.md §5 pins it:
      `All cells checked 2026-10-18 against the linked public sources.`
      Covers scenario "Dated line is verbatim".
- [x] 1.6 Add the `<details>` "Detailed comparison" block per design.md
      §6: five rows over the same six columns, subject cells from the
      research-backed values, Latest release and License / language
      competitor cells from task 1.2. Covers scenario "Release cells
      carry the linked shape".
- [x] 1.7 Add the positioning paragraph (exact text, design.md §7) and
      the also-exists lead-in plus five bullets (exact text, design.md
      §8), completing the section order: intro + legend, matrix,
      footnotes, dated line, details table, positioning paragraph,
      also-exists list. Covers scenarios "Section anatomy in fixed
      order", "Unique-combination claim only", and "Also-exists list
      names the five further tools".
- [x] 1.8 Neutrality pass over the whole section: grep it for
      evaluative adjectives ("better", "worse", "inferior", "best",
      "worst") — expect zero hits — and confirm every ❌/⚠️ cell states
      a fact linked to a source or covered by the legend's not-found
      rule. Covers scenario "Absence stated neutrally". Then run
      `make fmt` so prettier pads the new tables.

## 2. CONTRIBUTING re-audit rule (CONTRIBUTING.md)

- [x] 2.1 Append the exact bullet from design.md §10 to the
      `## Conventions` list — one sentence requiring a cell re-audit
      and a dated-line update for any edit touching the README
      `## Comparison` section. Add no bot, script, or workflow. Covers
      scenarios "Re-audit rule exists" and "Freshness mechanism stays
      manual".

## 3. CHANGELOG 0.4.0 bullet (CHANGELOG.md)

- [x] 3.1 Inside the existing `## 0.4.0` section, add a `### Added`
      subsection directly above `### Changed` holding exactly the one
      bullet from design.md §11. The heading stays `## 0.4.0`; no
      version changes anywhere in the file. No spec scenario — docs
      hygiene per the spec plan.

## 4. Verification

- [x] 4.1 Run `make fmt` then `make fmt-check`; expect exit 0 with no
      file listed as needing formatting (covers README.md,
      CONTRIBUTING.md, CHANGELOG.md, and the openspec markdown).
- [x] 4.2 Run `openspec validate add-readme-comparison`; expect VALID.
- [x] 4.3 No-version-change and scope check:
      `grep -n '^version' Cargo.toml herdr-plugin.toml` still prints
      `0.4.0` for both, and `git status --porcelain` lists only
      README.md, CONTRIBUTING.md, CHANGELOG.md, and files under
      openspec/changes/add-readme-comparison/ — no `src/`, `docs/`,
      `Cargo.lock`, or manifest change.

## 5. Remote/SSH row removal (post-review ruling)

The ruling recorded in proposal.md removes the Remote/SSH row from the
Comparison matrix entirely. The dated line stays
`All cells checked 2026-10-18 against the linked public sources.` — the
re-audit happens the same day — and the CONTRIBUTING rule wording itself
is unchanged.

- [x] 5.1 README.md — delete the Remote/SSH matrix row and the footnote
      definitions `[^20]`–`[^22]` (the subject's no-remote note,
      timofey-TK's SSH relay, alex-devdone's Remote-SSH); renumber the
      platform row's markers `[^23]`–`[^27]` to `[^20]`–`[^24]` and
      their definitions to match, so the footnote block holds 24
      entries in reading order per revised design.md §3–§4. Covers
      scenario "Seven criteria across six columns".
- [x] 5.2 README.md — delete the remote/SSH concession sentence (the
      paragraph's second sentence, conceding remote/SSH to the two
      named competitors) from the positioning paragraph; the paragraph
      becomes exactly the text pinned in revised design.md §7. Covers
      scenario "Unique-combination claim only".
- [x] 5.3 CHANGELOG.md — in the `## 0.4.0` `### Added` bullet, change
      "eight sourced criteria" to "seven sourced criteria" (revised
      design.md §11); no other CHANGELOG edit, no version change.
- [x] 5.4 Re-run task 1.8's neutrality grep over the section
      (evaluative adjectives — expect zero hits), then run `make fmt`
      and `make fmt-check`; expect exit 0. Covers scenario
      "Absence stated neutrally".
- [x] 5.5 Run `openspec validate add-readme-comparison`; expect VALID.
