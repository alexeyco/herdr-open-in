## MODIFIED Requirements

### Requirement: CONTRIBUTING carries the manual re-audit rule

CONTRIBUTING.md SHALL carry, under `## Conventions`, one rule sentence: an edit that touches the README `## Comparison` section re-audits every cell of both tables against the linked public sources and updates the `All cells checked …` date in the same change. This manual audit SHALL cover the factual meaning of every cell; the scheduled checker SHALL only verify configured release tags, release-cell presence, and HTTP(S) link liveness.

#### Scenario: Re-audit rule exists

- **WHEN** a contributor reads `## Conventions` in CONTRIBUTING.md
- **THEN** one bullet states that touching the README `## Comparison` section requires re-auditing the cells against the linked public sources and updating the `All cells checked …` date in the same change

#### Scenario: Manual review covers cell meaning

- **WHEN** a contributor changes a factual claim in either comparison table
- **THEN** the contributor re-audits every cell against its linked public source and updates the checked date in that change, regardless of the scheduled check result

#### Scenario: Freshness mechanism stays manual

- **WHEN** the repository is inspected for comparison-freshness enforcement
- **THEN** the scheduled workflow checks release tags, release-cell presence, and link liveness, while human review remains the only mechanism for checking the meaning of comparison claims

### Requirement: Monthly comparison freshness is checked automatically

The repository SHALL provide a GitHub Actions workflow in `.github/workflows/comparison-freshness.yml` with workflow display name `Comparison Freshness`. Its job SHALL use the key `comparison-freshness` and the job-level display name `Comparison Freshness`. It SHALL run at `0 6 1 * *` and support `workflow_dispatch`. The checker SHALL compare configured release tags with the upstream latest releases, verify that each HTTP(S) URL in the README comparison section is live, and verify the configured release cells are present. It SHALL NOT infer or alter the factual meaning of matrix or detailed-table cells, nor update the human checked date.

#### Scenario: Scheduled and manual execution are available

- **WHEN** the first day of a month arrives at 06:00 UTC or a maintainer dispatches the workflow manually
- **THEN** the `Comparison Freshness` workflow runs its `comparison-freshness` job against the README comparison

#### Scenario: Job has the requested display name

- **WHEN** a maintainer inspects the workflow's configured job
- **THEN** the job key is `comparison-freshness` and GitHub displays it as `Comparison Freshness`

#### Scenario: Clean comparison completes successfully

- **WHEN** configured release tags match, expected release cells are present, and all checked links are live
- **THEN** the checker reports a clean result and the workflow succeeds without changing the README or opening an issue or pull request

#### Scenario: Release-tag drift opens an update pull request

- **WHEN** a configured upstream latest release tag differs from its README release cell
- **THEN** the checker updates the README release cell and configured expected tag together and the workflow opens or updates a rolling pull request for review without writing directly to the base branch

#### Scenario: Other comparison drift is reported

- **WHEN** a source is reachable but a checked link is dead, moved, or an expected release cell is missing
- **THEN** the checker reports the drift through a single issue that it creates or updates, without silently rewriting the comparison claim

#### Scenario: Unavailable source fails safely

- **WHEN** a required upstream request cannot establish source availability
- **THEN** the workflow fails and creates neither a pull request nor an issue for that run
