# Phase 4 Review (`phase-4/error-handling`)

Compared `8bb0d69..023376d`.

## Findings (ordered by severity)

### 1) Important: `read_config()` still silently swallows distinct config failures
- Phase 4 intended differentiated handling/warnings for:
  - config path failure
  - malformed JSON
  - permission/IO issues
- Current code still falls back to defaults silently:
  - `config_path()` failure: immediate default return, no warning
  - read failure: `fs::read_to_string(...).unwrap_or_default()`
  - parse failure: `serde_json::from_str(...).unwrap_or_default()`

References:
- `/Users/hongzhekhooi/coingecko-cli-rs/src/config.rs:71`
- `/Users/hongzhekhooi/coingecko-cli-rs/src/config.rs:77`
- `/Users/hongzhekhooi/coingecko-cli-rs/src/config.rs:78`

### 2) Medium: HTTP client timeout/connect-timeout hardening from plan is not implemented
- `Client::build()` was correctly changed to return `Result`, but request timeouts were not added.
- No `.timeout(...)` or `.connect_timeout(...)` calls are present on the `reqwest::Client::builder()`.

Reference:
- `/Users/hongzhekhooi/coingecko-cli-rs/src/api.rs:26`

## What Looks Good

- Panicking `expect(...)` paths in config/client creation were removed and converted to `Result` propagation.
- `save_credentials` error handling is now surfaced in auth flow.
- CLI arg help text coverage for `History`, `Markets`, and `Search` was added.

## Verification Notes

- `cargo build` passed
- `cargo clippy -- -D warnings` passed
- `cargo test` passed (49 unit + 9 integration)
