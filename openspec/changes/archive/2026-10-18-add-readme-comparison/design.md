# Design — add-readme-comparison

Docs-only change. Files touched: `README.md`, `CONTRIBUTING.md`,
`CHANGELOG.md`, plus this change's artifacts under
`openspec/changes/add-readme-comparison/`. No file under `src/`,
`docs/`, `Cargo.toml`, `Cargo.lock`, or `herdr-plugin.toml` is touched;
all three version strings stay `0.4.0`.

## 1. Section placement

`## Comparison` goes between `## Usage` and `## Editor catalog` (the
catalog subsection `### Editor catalog` plus its detection paragraph
promoted to a top-level `##` section), followed by `## Development`.
Resulting `##` order: Requirements, Install, Usage, Comparison, Editor
catalog, Development, Changelog, License. This mirrors herdr-review,
where the comparison follows the short usage/quickstart block and
precedes the deep-dive sections.

## 2. Column lineup

Six columns, fixed order. Headers are links; each competitor header
displays a short tool name (the `herdr-`/`herdr-plugin-` prefix dropped),
and the two repos sharing the name `herdr-open-in-editor` display their
owner instead, which disambiguates.

| #   | Header text               | URL                                                       |
| --- | ------------------------- | --------------------------------------------------------- |
| 1   | `herdr-open-in` (subject) | `https://github.com/alexeyco/herdr-open-in`               |
| 2   | `timofey-TK`              | `https://github.com/timofey-TK/herdr-open-in-editor`      |
| 3   | `open-project-with`       | `https://github.com/harish-zuper/herdr-open-project-with` |
| 4   | `ide-jump`                | `https://github.com/agentience/herdr-plugin-ide-jump`     |
| 5   | `cursor-open`             | `https://github.com/alex-devdone/herdr-cursor-open`       |
| 6   | `upnorthmedia`            | `https://github.com/upnorthmedia/herdr-open-in-editor`    |

## 3. Matrix cell map

Seven rows, fixed order and wording. `f<n>` marks a cell carrying
footnote `[^n]`; footnotes are numbered in reading order (rows top to
bottom, cells left to right) and their definitions are listed in §4.
Bare cells carry no marker: subject ✅ cells are self-evident from this
README; bare ❌ cells fall under the legend's not-found rule.

| Criterion row (exact text)             | herdr-open-in | timofey-TK | harish-zuper | agentience | alex-devdone | upnorthmedia |
| -------------------------------------- | ------------- | ---------- | ------------ | ---------- | ------------ | ------------ |
| Opens the workspace via a herdr action | ✅            | ✅ f1      | ✅ f2        | ✅ f3      | ✅ f4        | ✅ f5        |
| Picker of choices at one keypress      | ✅            | ❌ f6      | ✅ f7        | ⚠️ f8      | ✅ f9        | ❌ f10       |
| Editors discovered, not preconfigured  | ✅            | ⚠️ f11     | ✅ f12       | ❌ f13     | ❌ f14       | ⚠️ f15       |
| Terminal editors run inside herdr      | ✅            | ❌         | ❌           | ❌         | ❌           | ❌           |
| JetBrains-family coverage              | ✅            | ❌         | ✅ f16       | ❌ f17     | ❌           | ⚠️ f18       |
| Remembers the last choice              | ✅            | ❌         | ❌           | ⚠️ f19     | ❌           | ❌           |
| Runs on both Linux and macOS           | ✅            | ✅ f20     | ❌ f21       | ❌ f22     | ❌ f23       | ❌ f24       |

Decisions resolved here (research left them open):

- **agentience row 2 is ⚠️, not ✅.** Its popup lists already-open IDE
  windows with the project match preselected — windows to raise, not
  editors to launch — which is the legend's "implemented differently".
  The research note "✅ popup of windows (⚠️ footnote)" is not one legend
  state; the user decision demands honest ⚠️ cells for the different
  mechanism.
- **upnorthmedia row 3 is ⚠️.** It does probe `PATH` then
  `/Applications`, but only within a preconfigured editor set
  (Zed/VS Code/Cursor/Sublime + custom template), so "discovered, not
  preconfigured" holds only partially.
- **Platform ❌ cells (row 7) carry footnotes**, not the bare-❌ rule:
  the macOS-only scope is positively documented, and the legend's ❌
  wording ("not found") would misstate a documented fact. Same for the
  documented no-picker designs (f6, f10) and the config-driven
  discovery absence (f13, f14).

## 4. Footnote definitions (exact texts, reading order)

Each definition ends with a source link. `[source](URL)` uses the column
repo URL from §2. The subject column carries no footnote: every subject
cell is a bare ✅, self-evident from this README.

1. `[^1]`: timofey-TK — installs as a herdr plugin; its actions open the workspace in VS Code or Zed — [source](https://github.com/timofey-TK/herdr-open-in-editor).
2. `[^2]`: harish-zuper — the herdr action opens the project in the app picked from the fzf popup (Finder, VS Code, IDEA, Android Studio, Xcode, Terminal) — [source](https://github.com/harish-zuper/herdr-open-project-with).
3. `[^3]`: agentience — the herdr action surfaces the workspace in an IDE by raising an already-open IDE window rather than launching an editor — [source](https://github.com/agentience/herdr-plugin-ide-jump).
4. `[^4]`: alex-devdone — the herdr action opens the workspace in Cursor or VS Code — [source](https://github.com/alex-devdone/herdr-cursor-open).
5. `[^5]`: upnorthmedia — the herdr action opens the workspace, or a file:line path the agent printed, in the configured editor — [source](https://github.com/upnorthmedia/herdr-open-in-editor).
6. `[^6]`: timofey-TK — no picker: the plugin picks automatically by preference and ships explicit per-editor actions (`open-vscode`, `open-zed`) — [source](https://github.com/timofey-TK/herdr-open-in-editor).
7. `[^7]`: harish-zuper — one keypress opens an fzf fuzzy popup over the apps discovered at popup open — [source](https://github.com/harish-zuper/herdr-open-project-with).
8. `[^8]`: agentience — the popup lists already-open IDE windows, preselecting the project match: windows to raise, not editors to launch — [source](https://github.com/agentience/herdr-plugin-ide-jump).
9. `[^9]`: alex-devdone — a popup offers the configured editor set under the keys 1 and 2 — [source](https://github.com/alex-devdone/herdr-cursor-open).
10. `[^10]`: upnorthmedia — `editor.conf` fixes one editor; the README states "no picker and nothing to type" — [source](https://github.com/upnorthmedia/herdr-open-in-editor).
11. `[^11]`: timofey-TK — probes the CLI on `PATH` only, with no macOS app-bundle fallback: a GUI editor without a runnable CLI is not offered — [source](https://github.com/timofey-TK/herdr-open-in-editor).
12. `[^12]`: harish-zuper — discovers the installed `.app` bundles when the popup opens — [source](https://github.com/harish-zuper/herdr-open-project-with).
13. `[^13]`: agentience — `config.json` names the app to raise; nothing is discovered from the machine — [source](https://github.com/agentience/herdr-plugin-ide-jump).
14. `[^14]`: alex-devdone — the editor set is configured, not discovered — [source](https://github.com/alex-devdone/herdr-cursor-open).
15. `[^15]`: upnorthmedia — probes `PATH` then `/Applications`, but only within the preconfigured editor set (Zed, VS Code, Cursor, Sublime, custom template) — [source](https://github.com/upnorthmedia/herdr-open-in-editor).
16. `[^16]`: harish-zuper — the discovered set includes IntelliJ IDEA and Android Studio — [source](https://github.com/harish-zuper/herdr-open-project-with).
17. `[^17]`: agentience — no JetBrains launch: an IDE is offered only when its window is already open and named in `config.json` — [source](https://github.com/agentience/herdr-plugin-ide-jump).
18. `[^18]`: upnorthmedia — no built-in JetBrains rows; a custom editor template can name any CLI — [source](https://github.com/upnorthmedia/herdr-open-in-editor).
19. `[^19]`: agentience — preselects the window matching the current project: a project match, not a remembered last choice — [source](https://github.com/agentience/herdr-plugin-ide-jump).
20. `[^20]`: timofey-TK — documented for Linux and macOS — [source](https://github.com/timofey-TK/herdr-open-in-editor).
21. `[^21]`: harish-zuper — documented as macOS-only (`.app` bundle discovery) — [source](https://github.com/harish-zuper/herdr-open-project-with).
22. `[^22]`: agentience — documented for macOS and Windows; no Linux — [source](https://github.com/agentience/herdr-plugin-ide-jump).
23. `[^23]`: alex-devdone — documented as macOS-only, implemented in bash — [source](https://github.com/alex-devdone/herdr-cursor-open).
24. `[^24]`: upnorthmedia — documented as macOS-only — [source](https://github.com/upnorthmedia/herdr-open-in-editor).

No evidence repeats across two cells, so no duplicated definition text
occurs; if the implementer adds a cell footnote, numbering stays in
reading order and duplicates get two identical definitions per the delta
spec.

## 5. Intro sentence, legend, dated line (exact texts)

- Intro: `How this plugin compares with the five other editor-launch tools that integrate with herdr, cell by cell as documented in their public sources.`
- Legend (verbatim from the playback): `✅ documented in the linked sources · ⚠️ partial or implemented differently · ❌ not found in the public sources checked`
- Dated line: `All cells checked 2026-10-18 against the linked public sources.`

## 6. Detailed table

`<details>` / `<summary>Detailed comparison</summary>`, five rows over
the same six columns; every cell carries a direct link (repo, release,
or tag URL), no footnotes. Row contents:

- **Editor selection** — subject: popup picker of the editors detected on this machine; timofey-TK: automatic by preference plus explicit per-editor actions; harish-zuper: fzf fuzzy popup of discovered apps; agentience: popup of open IDE windows, project match preselected; alex-devdone: popup keys 1/2 over the configured set; upnorthmedia: fixed editor from `editor.conf` ("no picker and nothing to type"). Each cell ends with `([repo](URL))`.
- **External runtime dependencies** — subject: none beyond herdr (Rust binary; `stty` child only); timofey-TK: python3; harish-zuper: fzf; agentience: python3; alex-devdone: none (bash); upnorthmedia: jq. Each with `([repo](URL))`.
- **Platforms** — subject: Linux, macOS; timofey-TK: Linux, macOS; harish-zuper: macOS; agentience: macOS, Windows; alex-devdone: macOS; upnorthmedia: macOS. Each with `([repo](URL))`.
- **Latest release** — subject: `[releases](https://github.com/alexeyco/herdr-open-in/releases)`, unpinned. Each competitor: `[<tag>](https://github.com/<owner>/<repo>/releases/tag/<tag>)` when a release exists, else `[no releases](https://github.com/<owner>/<repo>/releases)`.
- **License / language** — subject: `[MIT · Rust](https://github.com/alexeyco/herdr-open-in)`. Each competitor: `[<license> · <language>](https://github.com/<owner>/<repo>)`, or `no license file` in place of `<license>` when the repo publishes none.

The Latest-release tags and License/language values for the five
competitors are NOT in the 2026-10-18 research set. Task 1.2 fetches
them read-only from the linked repos (releases page, LICENSE file,
primary language) at implementation time; every other cell value above
is research-backed as of 2026-10-18, which is the date the dated line
records.

## 7. Positioning paragraph (exact text)

> Among the tools compared above, this plugin is the only one that
> combines a picker of the editors discovered on this machine, terminal
> editors (nvim, vim, helix) running in herdr tabs, the macOS app-bundle
> fallback for GUI editors whose CLI is missing, and last-choice memory.

Defensibility check against the matrix: harish-zuper has a picker (f7)
and discovery (f12) but no terminal editors and no last-choice memory;
agentience matches none of the four parts outright (⚠️ at best);
timofey-TK and upnorthmedia have no picker; alex-devdone has a popup but
a configured set and no terminal editors or memory. No competitor column
shows ✅ on all four rows 2, 3, 4, 6 — the combination claim holds and
no single-criterion uniqueness is claimed (rows 2 and 3 have other ✅
columns).

## 8. Also-exists list (exact text)

Lead-in line: `Also exists in this space, without a cell-by-cell comparison above:`

- `[herdr_plugin](https://github.com/Andreslvc/herdr_plugin) — opens VS Code, copies paths, handles spaces in paths.`
- `[herdr-positron](https://github.com/gadenbuie/herdr-positron) — opens Positron; POSITRON_CMD overrides the binary.`
- `[herdr-open](https://github.com/kongdd/herdr-open) — two fixed actions for Zed and VS Code; SSH relay; jq dependency.`
- `[herdr-launcher-pane](https://github.com/y-hirakaw/herdr-launcher-pane) — workspace-by-command dock with a mouse UI; Finder and VS Code built in.`
- `[hither](https://github.com/T0mSIlver/hither) — remote pane; opens a Zed SSH project on macOS via launchd.`

## 9. Cross-link fix (README.md:9-12, exact result)

`Other herdr plugins:` list becomes exactly two entries, alphabetical,
mirroring herdr-review's own list (which links this plugin and
herdr-gitui):

- `- [herdr-gitui](https://github.com/alexeyco/herdr-gitui) — open gitui in a new tab and report what the session changed to the agent in the focused pane.` (existing entry, kept)
- `- [herdr-review](https://github.com/alexeyco/herdr-review) — review diffs with the human review TUI of your choice and deliver your annotations to the agent in the focused pane.`

The herdr-revdiff entry is deleted.

## 10. CONTRIBUTING rule (exact bullet, appended to `## Conventions`)

`- An edit that touches the README \`## Comparison\` section re-audits every cell of both tables against the linked public sources and updates the \`All cells checked …\` date in the same change.`

One sentence, one bullet. No bot, no script, no workflow — the rule is
the whole freshness mechanism per the playback.

## 11. CHANGELOG entry (exact shape)

Inside the existing `## 0.4.0` section (heading unchanged), add a new
`### Added` subsection directly above `### Changed`, holding exactly one
bullet:

`- Added a \`## Comparison\` section to the README: this plugin versus the five other herdr editor-launch plugins across seven sourced criteria, with a detailed table, an audit date, and an also-exists list of related tools.`

## 12. Formatting

`make fmt-check` runs `npx --yes prettier --check "**/*.md"`, which
covers README.md, CONTRIBUTING.md, CHANGELOG.md, and the openspec
markdown. The matrix and detailed tables will be wide; write them with
plain single-space pipes and run `make fmt` once so prettier applies its
own padding — do not hand-pad. Checkmark cells keep the emoji +
variation selector exactly as in the legend line.
