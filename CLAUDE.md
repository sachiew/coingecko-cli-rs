# CLAUDE.md — coingecko-cli-rs

## Quick Reference

```bash
cargo build                          # compile (dev)
cargo build --release                # compile (release)
cargo test --all-targets             # run all tests (49 unit + 9 integration)
cargo clippy -- -D warnings          # lint (pedantic enabled, must pass clean)
cargo fmt --check                    # check formatting
cargo fmt                            # auto-format
```

## Project Overview

A Rust CLI tool for querying the CoinGecko API. Installed as `cg` (primary) or `coingecko` (alias). Both bin targets share `src/main.rs`.

**Version**: 1.3.0 — managed in `Cargo.toml`, referenced at runtime via `env!("CARGO_PKG_VERSION")`.

### Commands

| Command | Description |
|---------|-------------|
| `cg` | Branded landing screen |
| `cg auth` | Save API key and tier (demo/pro) interactively or via `--key`/`--tier` flags |
| `cg status` | Show current auth configuration |
| `cg price --ids bitcoin,ethereum` | Current prices (supports `--symbols btc,eth` for auto-resolution) |
| `cg markets --total 100` | Top coins by market cap (supports `--category`, `--export`, `--order`) |
| `cg search ethereum` | Search coins, exchanges, categories |
| `cg trending` | Trending coins, NFTs, and categories (24h) |
| `cg history bitcoin --days 30` | Historical price data (also `--date YYYY-MM-DD`, `--from`/`--to` range) |
| `cg tui` | Interactive TUI browser (top 50 coins, ratatui) |
| `cg tui-trending` | Interactive TUI browser (top 30 trending coins) |

**Global flag**: `--json` outputs machine-readable JSON on all data commands.

## Architecture

```
src/
  main.rs          CLI definition (clap derive), entry point, auth/status commands
  config.rs        Persistent config — API key + tier via OS config directory
  ui.rs            Terminal formatting — colors, logo, welcome box, USD/change formatters
  tui.rs           Interactive TUI mode — ratatui market and trending browser
  api/
    mod.rs         Module declarations + pub re-exports (public API surface)
    client.rs      HTTP client — Client struct, build(), get() with auth headers
    date.rs        Date utility functions (ymd_to_unix, unix_to_ymd, etc.) + tests
    price.rs       `cg price` command handler
    trending.rs    `cg trending` command handler (3 tables: coins, NFTs, categories)
    markets.rs     `cg markets` command handler + MarketCoin struct + CSV export
    search.rs      `cg search` command handler + SearchResponse/SearchCoin structs
    history.rs     `cg history` command handler + chart display + ChartData struct
    tui_data.rs    TUI data fetchers — MarketEntry, CoinDetail, fetch_* functions
tests/
  cli.rs           Integration tests — deterministic CLI parsing checks (no network)
```

### Module Dependency Flow

```
main.rs ──→ api::run_*()        (command dispatch)
        ──→ config              (auth/status commands)
        ──→ ui                  (banner, formatting)

tui.rs  ──→ api::{MarketEntry, CoinDetail, fetch_*()}
        ──→ ui                  (formatting)

api/* modules ──→ api::client   (HTTP requests)
              ──→ api::date     (history.rs only)
              ──→ ui            (table formatting)
              ──→ config        (client.rs only, for credentials)
```

## Code Patterns

### Error Handling

- All command handlers (`run_*`) return `Result<(), Box<dyn std::error::Error>>`
- All data fetchers (`fetch_*`) return `Result<T, Box<dyn std::error::Error>>`
- `main()` catches errors with `if let Err(e) = ...`, prints to stderr, and calls `std::process::exit(1)`
- Config reads fall back to defaults with `eprintln!` warnings on failures (never panics)
- No `expect()` or `unwrap()` in production paths — all converted to `?` or `.unwrap_or_default()`

### HTTP Client

- `Client` in `api/client.rs` is `pub(crate)` — shared by all API modules
- `Client::build()` reads credentials from config, sets user-agent, connect timeout (10s), request timeout (30s)
- `Client::get(path)` constructs authenticated requests with the correct header (`x-cg-demo-api-key` or `x-cg-pro-api-key`)
- API errors return `Err(format!("API error {status}: {body}").into())`

### `--json` Contract

- When `--json` is active, **only** valid JSON goes to stdout
- All diagnostics, warnings, and export messages go to stderr (`eprintln!`)
- `--json` + `--export` both apply simultaneously (CSV to file AND JSON to stdout)
- Empty results output valid empty JSON (`[]` or `{}`)
- `print_banner()` is suppressed when `--json` is active

### Cross-Module Visibility

- `api/mod.rs` re-exports the public surface — `main.rs` and `tui.rs` use `api::run_*` and `api::fetch_*`
- `MarketCoin` (in `markets.rs`) is `pub(crate)` with all fields `pub(crate)` — consumed by `tui_data.rs`
- `ChartData` (in `history.rs`) is `pub(crate)` with `prices` field `pub(crate)` — consumed by `tui_data.rs`
- Private structs stay private within their modules (e.g., `SearchResponse`, `HistoryPoint`)

### Clippy & Lints

- Clippy pedantic enabled in `Cargo.toml` with targeted exceptions (`module_name_repetitions`, `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`)
- Function-level `#[allow]` for justified cases:
  - `too_many_lines` on `run_trending`, `run_markets`, `display_chart`, `run_history`
  - `too_many_arguments` on `run_history` (8 params across 3 date modes)
  - `cast_possible_truncation` on `ms_to_date` (f64 → i64 millisecond conversion)

### Testing

- **Unit tests** (49): inline `#[cfg(test)] mod tests` in `date.rs`, `config.rs`, `ui.rs`
- **Integration tests** (9): `tests/cli.rs` — deterministic CLI parsing, no network calls
- Test binary resolved via `env!("CARGO_BIN_EXE_cg")` (no external test dependencies)
- No API mocking — tests cover pure/deterministic functions only

### Config Storage

- Config path: OS config directory via `directories::ProjectDirs` → `coingecko-cli/config.json`
- Stored as JSON: `{ "api_key": "...", "tier": "demo"|"pro" }`
- `Tier` enum: `Demo` (api.coingecko.com) vs `Pro` (pro-api.coingecko.com)
- `read_config()` warns to stderr on failures, never panics

## CI

GitHub Actions (`.github/workflows/ci.yml`):
- Runs on push to `main` and PRs
- Matrix: `ubuntu-latest` + `macos-latest`
- Steps: `fmt --check` → `clippy -D warnings` → `build` → `test --all-targets`
- Separate `audit` job via `rustsec/audit-check`
