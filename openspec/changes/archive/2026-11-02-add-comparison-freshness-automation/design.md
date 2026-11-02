# Design: comparison freshness workflow

## Approach

Mirror the successful `herdr-git` workflow pattern, scoped to this repository's README comparison. A Node.js checker reads `README.md` and the configured peer/release expectations from `.github/workflows/comparison-freshness.yml`. Check mode classifies release-tag drift, non-fixable link/cell drift, and unavailable sources. Fix mode updates only release cells and their expected tags. The workflow routes fixable release drift to a rolling pull request, routes other drift to an issue, and fails on unavailable sources before creating either artifact.

The workflow file uses the stable slug path `.github/workflows/comparison-freshness.yml` and workflow display name `Comparison Freshness`. Its job key is `comparison-freshness` and its job-level display name is `Comparison Freshness`. It runs at `0 6 1 * *` and on `workflow_dispatch`. Repository permissions are limited to contents, pull requests, and issues required by the PR and issue actions. Concurrency prevents overlapping monthly and manual runs from racing.

The checker does not claim to audit whether a comparison criterion is true. Contributors continue to review both tables' factual cells and update the checked date when comparison content changes. The scheduled checker does not change that date.

## Files to touch

- `.github/workflows/comparison-freshness.yml` — define schedule, manual dispatch, job name, peer release expectations, permissions, drift routing, and fail behavior.
- `scripts/check-comparison.mjs` — implement check, release-cell fix, and issue-report modes for this README's comparison.
- `tests/check-comparison.test.mjs` — test clean, drift, fix, issue, and unavailable-source outcomes using controlled HTTP responses and temporary fixture files.
- `README.md` — manually re-audit both comparison tables and refresh the checked date; include current release tags and source facts.
- `CONTRIBUTING.md` — retain the human cell re-audit rule and explain the scope of the scheduled checks.
- `openspec/specs/public-positioning/spec.md` — sync the revised freshness requirements after implementation.

## Data and failure handling

The workflow's peer/release expectation list is the source of expected tags. Derive its peer set from the release rows audited in `README.md` and keep that set documented in `.github/workflows/comparison-freshness.yml`; audit the exact peer set against the current README release rows during implementation. A missing or malformed expectation is an unavailable configuration finding. Release tags are obtained from GitHub's latest-release endpoint. README URLs are checked with redirects followed. Transport failures, rate limits, and server errors receive one retry; an unresolved request is unavailable. Unavailable findings dominate drift findings and fail the run without creating a PR or issue.

The release fixer prepares the README and workflow edits before writing either file, and only changes matching release cells and expected tag entries. Link drift and missing release cells cannot be safely reconstructed, so they are written to the drift report used to create or update the issue. A release-tag pull request never writes directly to the base branch.

## Trade-offs

- Monitoring every linked URL catches dead or moved sources, while source content remains subject to human interpretation.
- Updating only the release row avoids presenting an automated edit as a fresh audit of all comparison claims.
- The first-of-month UTC schedule gives a stable monthly cadence independent of local time changes; manual dispatch supports an immediate recheck.
