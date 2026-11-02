# Tasks: Automate monthly README comparison freshness checks

## 1. Audit and refresh the comparison

- [x] 1.1 Re-audit both README comparison tables against their linked public sources; update factual cells, release and license/language details, footnotes if needed, and the checked date. Confirm the seven matrix criteria and six columns remain intact.

## 2. Implement the checker and workflow

- [x] 2.1 Add `scripts/check-comparison.mjs` with check, release-tag fix, and drift-report modes; distinguish clean, fixable release drift, other drift, and unavailable source outcomes. Match the scenarios “Clean comparison completes successfully”, “Release-tag drift opens an update pull request”, “Other comparison drift is reported”, and “Unavailable source fails safely”.
- [x] 2.2 Add `tests/check-comparison.test.mjs` with HTTP and filesystem fixtures for each checker classification and verify unavailable-source outcomes do not modify files or request PR/issue creation. Run the tests and record their result.
- [x] 2.3 Add `.github/workflows/comparison-freshness.yml` with workflow name `Comparison Freshness`, job key `comparison-freshness`, job-level display name `Comparison Freshness`, monthly cron `0 6 1 * *`, `workflow_dispatch`, least required repository permissions, and non-overlapping concurrency. Derive the peer set from the audited README release rows and document it in the workflow. Match the scenarios “Scheduled and manual execution are available” and “Job has the requested display name”.
- [x] 2.4 Route release-tag drift to a rolling pull request and non-fixable drift to a create-or-update issue; fail unavailable-source runs without either artifact. Verify the base branch is never modified directly.

## 3. Update contributor guidance and verify

- [x] 3.1 Update `CONTRIBUTING.md` under `## Conventions` to retain the manual re-audit/date rule and describe the checker's limited coverage. Match the scenarios “Re-audit rule exists”, “Manual review covers cell meaning”, and “Freshness mechanism stays manual”.
- [x] 3.2 Run the comparison checker against the audited README and the project-defined formatting checks for changed Markdown and workflow files; verify all scenarios in this change and report command output.
- [x] 3.3 Sync `openspec/specs/public-positioning/spec.md` with the implemented workflow, checker, and manual review boundary.
