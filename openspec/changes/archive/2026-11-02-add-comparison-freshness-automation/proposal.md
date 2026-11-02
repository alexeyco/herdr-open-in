# Automate monthly README comparison freshness checks

## Why now

The README comparison includes dated claims and links to public sources. The current `public-positioning` specification requires contributors to re-audit the comparison manually and explicitly disallows a bot or script. A monthly workflow can detect stale release tags and unavailable or changed source links, while a human remains responsible for re-auditing the meaning of comparison cells.

## What changes

- Update the README comparison table after a manual review of its linked sources, including the detailed release and license/language rows and its checked date.
- Add a monthly GitHub Actions workflow named `Comparison Freshness`, stored at `.github/workflows/comparison-freshness.yml`, with a `comparison-freshness` job id, a first-of-month 06:00 UTC schedule, and manual dispatch.
- Add a checker that verifies configured peers' latest release tags, checks the README's HTTP(S) links, and confirms the expected release cells are present. For release-tag drift, update the README release cells and configured expected tags together and open or update a rolling pull request. Route non-fixable source drift to an issue. Fail when a source is unavailable, without opening a pull request or issue.
- Revise `public-positioning` and the contributor guidance so automated checks cover link liveness and release-tag freshness, while contributors continue to re-audit factual cells and update the checked date when they edit the comparison.

## Impact on capabilities

| Capability           | Impact                                                                                                              |
| -------------------- | ------------------------------------------------------------------------------------------------------------------- |
| `public-positioning` | Modified: add monthly automated checks and drift handling while preserving manual review of cell meaning and dates. |

## Assumptions and accepted risks

ASSUMPTION: derive the monitored peer set from the release rows after the manual README audit and document the resulting list in `.github/workflows/comparison-freshness.yml`; audit the exact peer set against the current README during implementation. Check every HTTP(S) URL in the comparison section for liveness.

REJECTED-RISK: none.
