# Phase 3 Review (`phase-3/unit-tests`)

Compared `d05741f..19646bf`.

## Findings (ordered by severity)

### 1) Important: integration test binary lookup uses deprecated `assert_cmd::cargo::cargo_bin`
- `tests/cli.rs` uses a deprecated helper that is documented as incompatible with custom Cargo build dirs.
- This already emits warnings during `cargo test`, and can become a CI portability issue depending on runner/build-dir setup.

References:
- `/Users/hongzhekhooi/coingecko-cli-rs/tests/cli.rs:3`
- `/Users/hongzhekhooi/coingecko-cli-rs/tests/cli.rs:7`

Recommendation:
- Switch to `assert_cmd::cargo::cargo_bin!` macro (or `Command::cargo_bin` API) to remove deprecation and improve reliability.

### 2) Medium: deterministic CLI integration coverage is incomplete vs planned scope
- Added tests cover `--help`, `--version`, `history --help`, `price --help`, and invalid subcommand.
- Missing checks that were part of the phase plan intent:
  - `markets --help` deterministic check
  - bare `cg` success-path check

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/tests/cli.rs`

Recommendation:
- Add those two tests to fully satisfy planned deterministic CLI contract coverage.

## Verification Notes

- `cargo build` passed
- `cargo clippy -- -D warnings` passed
- `cargo test` passed (all tests green)
- `cargo test` output includes deprecation warnings from `assert_cmd::cargo::cargo_bin`.
