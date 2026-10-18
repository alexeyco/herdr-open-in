# Add a sourced README Comparison section

## Why now

Six same-purpose herdr plugins — tools that open the workspace in an
external editor — were verified from public sources on 2026-10-18, enough
for a cell-by-cell comparison where every cell is backed by a linked
source. The sibling plugin herdr-review already ships such a
`## Comparison` section (its README.md:42-108 is the model) and already
cross-links to this plugin, while this README has no comparison and its
"Other herdr plugins" list (README.md:9-12) still links the stale
herdr-revdiff. Without the section, a reader cannot see what this plugin
does that no compared competitor does — a picker of editors detected from
the machine, terminal editors running in herdr tabs, the macOS app-bundle
detection fallback, and last-choice memory.

## What changes

- **README.md — new `## Comparison` section** placed between `## Usage`
  and `## Development`, modeled on herdr-review: intro sentence, legend
  line (`✅ documented in the linked sources · ⚠️ partial or implemented
differently · ❌ not found in the public sources checked`), a
  7-criteria × 6-column matrix (this plugin plus timofey-TK/herdr-open-in-editor,
  harish-zuper/herdr-open-project-with, agentience/herdr-plugin-ide-jump,
  alex-devdone/herdr-cursor-open, upnorthmedia/herdr-open-in-editor),
  numbered footnotes carrying the source links, the dated line
  `All cells checked 2026-10-18 against the linked public sources.`, a
  `<details>` detailed table (editor selection, external runtime
  dependencies, platforms, latest release, license / language), a
  positioning paragraph claiming only the defensible unique combination,
  and an "also exists" one-line list of five further tools
  (Andreslvc/herdr_plugin, gadenbuie/herdr-positron, kongdd/herdr-open,
  y-hirakaw/herdr-launcher-pane, T0mSIlver/hither).
- **README.md — cross-link fix.** The "Other herdr plugins" list
  (README.md:9-12) drops the stale herdr-revdiff entry and becomes
  herdr-gitui + herdr-review, mirroring herdr-review's mutual list.
- **CONTRIBUTING.md — one rule bullet** under `## Conventions`: an edit
  touching the README `## Comparison` section re-audits every cell
  against the linked public sources and updates the `All cells checked …`
  date in the same change. This rule is the whole freshness mechanism —
  no bot, no script.
- **CHANGELOG.md — one bullet** in a new `### Added` subsection of the
  existing unreleased `## 0.4.0` section, describing the README
  comparison section.
- **New capability spec delta.** `public-positioning` (ADDED, five
  requirements) mirroring the herdr-review capability of the same name,
  scoped to this repo and without its freshness-bot requirements.

Not changing: any file under `src/`, `Cargo.toml`, `Cargo.lock`,
`herdr-plugin.toml`, or `docs/`; the version (0.4.0 stays — it is
unreleased and this batch rides with it); plugin behavior; no remote/SSH
implementation; no comparison-freshness bot or script; no 10-column
matrix.

## Impact on capabilities

| Capability                                                                                                                                                                            | Impact                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `public-positioning`                                                                                                                                                                  | ADDED (new): README carries a sourced dated Comparison section; every factual cell sourced and legend-defined; neutral tone with only the defensible combination claim; CONTRIBUTING re-audit rule; cross-link list accuracy. |
| `plugin-registration`, `herdr-pane-control`, `editor-catalog`, `editor-detection`, `editor-launch`, `editor-picker`, `last-choice-state`, `terminal-editor-exec`, `workspace-context` | Unchanged. Docs-only change: no code, manifest, or version literal moves; `plugin-registration` keeps saying `0.4.0`.                                                                                                         |

## Assumptions and accepted risks

ASSUMPTION: the batch rides with the unreleased 0.4.0 — the user explicitly requires no version increment anywhere, so `Cargo.toml`, `herdr-plugin.toml`, and the CHANGELOG heading stay `0.4.0` and the `plugin-registration` spec's version literals are not touched; current state verified read-only: `grep -n '^version' Cargo.toml herdr-plugin.toml → Cargo.toml:3:version = "0.4.0"` and `→ herdr-plugin.toml:4:version = "0.4.0"`.
ASSUMPTION: competitor Latest-release tags and License/language facts are not in the 2026-10-18 research set (untested — task 1.2 fetches them read-only from the linked repos at implementation time; a repo with no published release or no license file takes the `no releases` / `no license file` cell shapes defined in design.md, which is what an unfetched or absent value would disprove by forcing a different shape).
ASSUMPTION: agentience/herdr-plugin-ide-jump's picker-row cell is ⚠️, not ✅ — the research note reads "✅ popup of windows (⚠️ footnote: windows, not editors)", which is not one legend state, and the user decision demands honest ⚠️ cells for its different mechanism (it raises already-open IDE windows rather than launching an editor); ⚠️ "implemented differently" is the resolvable reading (untested — a repo README explicitly documenting an editor-launch picker would disprove it).
REJECTED-RISK: none — the user dismissed no risk during the grill. The unfavorable fact kept on purpose (agentience/herdr-plugin-ide-jump as a column with honest ⚠️ cells) is an explicit user decision, not a dismissed risk; the second fact originally kept on purpose — this plugin's Remote/SSH ❌ row — was superseded by the ruling below, which removes that row from the matrix entirely.
ruling: remove the Remote/SSH row from the Comparison matrix entirely — “честно говоря, я никогда и не стремился к этому” (the plugin never targeted remote/SSH; the row implies a missed goal). This supersedes the earlier grill decision that kept the unfavorable cell.
