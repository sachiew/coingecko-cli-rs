# CLAUDE.md — coingecko-cli-rs

## Build & Run

```bash
cargo build              # compile
cargo run -- price --ids bitcoin   # run via cargo
cg price --ids bitcoin   # run installed binary
```

## Test / Lint

```bash
cargo test               # run all tests
cargo clippy -- -D warnings   # lint (pedantic enabled)
cargo fmt --check        # check formatting
cargo fmt                # auto-format
```

## Architecture

Single-binary CLI built with clap (derive). Two binaries (`cg` and `coingecko`) from the same source.

| File | Role |
|------|------|
| `src/main.rs` | CLI definition (clap), entry point, auth/status commands |
| `src/api.rs` | HTTP client, all CoinGecko API calls, data structs, date helpers |
| `src/config.rs` | Persistent config (API key, tier) via OS config directory |
| `src/ui.rs` | Terminal formatting: colors, logo, welcome box, USD/change formatters |
| `src/tui.rs` | Interactive TUI mode (ratatui) for markets and trending |

## Key Patterns

- All API functions return `Result<(), Box<dyn std::error::Error>>`
- Config stored as JSON in OS config dir via `directories` crate
- `--json` flag outputs machine-readable JSON to stdout; all diagnostics go to stderr
- Clippy pedantic lints are enabled in `Cargo.toml`
