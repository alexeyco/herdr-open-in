# Contributing

## Setup

- Install [rustup](https://rustup.rs); the pinned toolchain comes from `rust-toolchain.toml` on first build.
- herdr >= 0.9.0 for linking and testing.

## Checks

What CI runs, and what should pass before you push: see [docs/development.md](docs/development.md) → Checks.

Format Markdown with `make fmt`; CI enforces the formatting check.

## Conventions

- Branch names use `<type>/<kebab-desc>` (for example, `refactor/simplify-editor-picker`).
- No `unwrap`/`expect` in production paths; errors are `Result<(), String>`.
- `#[allow(...)]` only with a comment explaining why.
- Conventional, lowercase commit messages (`feat: ...`, `fix: ...`); no AI co-author trailers.
- Run the checks in [docs/development.md](docs/development.md) → Checks before pushing.
- An edit that touches the README `## Comparison` section re-audits every cell of both tables against the linked public sources and updates the `All cells checked …` date in the same change.

## Workflow

1. Open an issue first for behavior changes (bug report or proposal templates).
2. Keep PRs focused; the PR template has a checklist.

## Releases

Maintainers tag `v<version>`; see [docs/development.md](docs/development.md) for the version sync rules and what the release workflow does.
