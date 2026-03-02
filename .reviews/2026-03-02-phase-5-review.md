# Phase 5 Review (`phase-5/ci-pipeline`)

Compared `b940d25..992636f`.

## Findings (ordered by severity)

### 1) Important: CI coverage is narrower than planned scope (single OS only)
- Workflow runs only on `ubuntu-latest`.
- The updated plan targeted a Linux + macOS matrix for broader compatibility checks.

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/.github/workflows/ci.yml:11`

### 2) Important: security audit job is missing
- The updated plan included an `audit` job (`rustsec/audit-check`) but this workflow defines only `check`.

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/.github/workflows/ci.yml`

### 3) Medium: test command does not include all targets
- Workflow runs `cargo test` instead of `cargo test --all-targets`.
- Current project still has two bin targets; `--all-targets` gives fuller coverage consistency.

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/.github/workflows/ci.yml:21`

## What Looks Good

- Core CI quality gates are present: `fmt --check`, `clippy -D warnings`, `build`, and `test`.
- Toolchain and cache setup are correct and conventional.
