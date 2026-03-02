# Phase 2 Review (`phase-2/json-output`)

Compared `23c80fa..3242e80` (Phase 2 commit).

## Findings (ordered by severity)

### 1) Important: `--json` mode does not produce machine-readable error output and still exits successfully on API failure paths
- In multiple `run_*` functions, non-success API responses are printed to stderr and return `Ok(())` instead of `Err(...)`:
  - `run_price` at `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:92`
  - `run_trending` at `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:183`
  - `run_markets` at `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:558`
  - `run_search` at `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:706`
  - `run_history` at `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:1146`
- In `main`, only `Err` is surfaced (`/Users/hongzhekhooi/coingecko-cli-rs/src/main.rs:140`), so these API failures still exit with code 0.
- Result: `--json` consumers can get empty stdout + human stderr + success exit code, which is not a reliable machine contract.

### 2) Medium: `search --json` shape differs from the planned/object style and is undocumented
- JSON mode returns a bare array (`Vec<&SearchCoin>`) rather than an object envelope:
  - `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:718`
- If downstream users expect stable per-command schemas, this shape mismatch can cause breakage.

## Verification Notes

- `cargo build` passed
- `cargo clippy -- -D warnings` passed
- `cargo test` passed (0 tests)
- Runtime probe:
  - `./target/debug/cg search --json bitcoin` produced stderr error text, zero stdout bytes, and exit code `0` in this environment.
