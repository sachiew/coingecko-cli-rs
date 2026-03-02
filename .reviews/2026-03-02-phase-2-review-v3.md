# Phase 2 Review (V3) (`phase-2/json-output`)

Compared `29893ba..d05741f` (exit-code fix).

## Findings

No new blocking findings in this delta.

## Resolved Since V2

- Command error paths now exit non-zero from `main()` via `std::process::exit(1)` in each `Err` dispatch branch.
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:141`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:151`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:190`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:200`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:229`

## Verification Notes

- `cargo build` passed
- `cargo clippy -- -D warnings` passed
- `cargo test` passed (0 tests)
- Runtime probe:
  - `./target/debug/cg search --json bitcoin` now exits with code `1` on request failure and writes error to stderr.

## Residual Risk

- There are still no automated tests (`cargo test` runs 0 tests), so the new `--json` behavior and exit-code contract are not regression-protected yet.
