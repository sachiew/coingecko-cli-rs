# Phase 2 Review (V2) (`phase-2/json-output`)

Compared `3242e80..29893ba` (follow-up fix commit).

## Findings (ordered by severity)

### 1) Important: runtime/network failures still return process exit code `0`
- `run_*` functions now return `Err(...)` on non-2xx HTTP status, which is good.
- But `main()` still catches errors and only prints to stderr without exiting non-zero:
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:140`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:150`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:177`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:195`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:212`
- Verified behavior: `./target/debug/cg search --json bitcoin` printed error on stderr and exited `0` in this environment.
- This still breaks machine-usage expectations for `--json` flows, because command failure is not reflected in the exit status.

## Resolved Since Previous Review

- `search --json` now returns a truncated `SearchResponse` object instead of a bare array:
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:700`
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:703`

## Verification Notes

- `cargo build` passed
- `cargo clippy -- -D warnings` passed
- `cargo test` passed (0 tests)
