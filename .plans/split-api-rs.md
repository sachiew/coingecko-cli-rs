# Plan: Split `api.rs` into Submodules

## Context

`src/api.rs` is 1397 lines containing the HTTP client, 7 date utility functions, 5 command handlers, 4 TUI data fetchers, 10 data structs, and 21 tests — all in one file. Splitting it into focused submodules improves readability and makes each concern independently navigable.

---

## Target Structure

```
src/api/
  mod.rs        — re-exports only (public API surface unchanged)
  client.rs     — Client struct, build(), get()
  date.rs       — date utility functions + their tests
  price.rs      — run_price
  trending.rs   — run_trending, change_cell helper
  markets.rs    — run_markets, MarketCoin, export_markets_csv
  search.rs     — run_search, SearchResponse, SearchCoin
  history.rs    — run_history, display_chart, chart types, export_chart_csv
  tui_data.rs   — MarketEntry, CoinDetail, fetch_* functions
```

---

## Steps

### 1. Rename `src/api.rs` → `src/api/mod.rs`

Rust module convention. No code changes — just the file move.

### 2. Extract `client.rs`

Move:
- `struct Client` + `impl Client { build(), get() }`
- `use crate::config::get_credentials` import

Make `Client` and methods `pub(crate)` so sibling submodules can use it.

### 3. Extract `date.rs`

Move:
- `is_leap()`, `days_in_month()`, `ymd_to_unix()`, `unix_to_ymd()`, `parse_ymd()`, `to_api_date()`, `ms_to_date()`
- Preserve `#[allow(clippy::cast_possible_truncation)]` on `ms_to_date`
- All 21 date-related tests from `#[cfg(test)] mod tests`

Make functions `pub(crate)`.

### 4. Extract `price.rs`

Move:
- `pub async fn run_price(...)`

Imports: `super::client::Client`, `crate::ui::{dim, format_usd}`, `HashMap`, `Value`, `comfy_table`

### 5. Extract `trending.rs`

Move:
- `fn change_cell(...)` (private helper)
- `pub async fn run_trending(...)` — preserve `#[allow(clippy::too_many_lines)]`

Imports: `super::client::Client`, `crate::ui::{dim, format_usd, format_large_usd, green_bold}`, `comfy_table`, `Value`

### 6. Extract `markets.rs`

Move:
- `struct MarketCoin` — make `pub(crate)` with all fields `pub(crate)` (consumed by `tui_data.rs::fetch_top_coins` which reads every field)
- `fn export_markets_csv(...)`
- `pub async fn run_markets(...)` — preserve `#[allow(clippy::too_many_lines)]`

Imports: `super::client::Client`, `crate::ui::{dim, format_usd, format_large_usd}`, `comfy_table`, `serde`, `csv`

### 7. Extract `search.rs`

Move:
- `struct SearchResponse`, `struct SearchCoin` (keep `Serialize` derives)
- `pub async fn run_search(...)`

Imports: `super::client::Client`, `crate::ui::dim`, `comfy_table`, `serde`

### 8. Extract `history.rs`

Move:
- `struct HistoryPoint`, `struct HistoryMarketData`
- `struct ChartData` — make `pub(crate)` with `prices` field `pub(crate)` (read by `tui_data.rs::fetch_coin_chart`)
- `fn export_chart_csv(...)`
- `fn display_chart(...)` — preserve `#[allow(clippy::too_many_lines)]`
- `pub async fn run_history(...)` — preserve `#[allow(clippy::too_many_lines, clippy::too_many_arguments)]`

Imports: `super::client::Client`, `super::date::*`, `crate::ui::{dim, format_usd, format_large_usd, format_change}`, `comfy_table`, `serde`, `csv`, `HashMap`, `Value`

### 9. Extract `tui_data.rs`

Move:
- `pub struct MarketEntry`, `pub struct CoinDetail`
- `struct CoinDetailRaw`, `struct CoinDetailMarketData` (private)
- `pub async fn fetch_top_coins(...)`, `fetch_coin_detail(...)`, `fetch_trending_coins(...)`, `fetch_coin_chart(...)`

Imports: `super::client::Client`, `super::markets::MarketCoin`, `super::history::ChartData`, `serde`, `Value`

### 10. Write `mod.rs` (final form)

```rust
//! `CoinGecko` API client — HTTP requests, response types, and data formatting.

mod client;
mod date;
mod history;
mod markets;
mod price;
mod search;
mod trending;
mod tui_data;

pub use history::run_history;
pub use markets::run_markets;
pub use price::run_price;
pub use search::run_search;
pub use trending::run_trending;
pub use tui_data::{fetch_coin_chart, fetch_coin_detail, fetch_top_coins, fetch_trending_coins};
pub use tui_data::{CoinDetail, MarketEntry};
```

### 11. No external changes needed

- `src/main.rs` — uses `api::run_*` — unchanged (re-exported from `mod.rs`)
- `src/tui.rs` — uses `api::{MarketEntry, CoinDetail, fetch_*}` — unchanged (re-exported)

---

## Approximate File Sizes After Split

| File | ~Lines | Content |
|------|--------|---------|
| `mod.rs` | 15 | Declarations + re-exports |
| `client.rs` | 35 | HTTP client |
| `date.rs` | 215 | 7 date functions + 21 tests |
| `price.rs` | 115 | Price command |
| `trending.rs` | 240 | Trending command |
| `markets.rs` | 195 | Markets command + CSV export |
| `search.rs` | 95 | Search command |
| `history.rs` | 310 | History command + chart display |
| `tui_data.rs` | 185 | TUI data fetchers |

---

## Extraction Notes

- Preserve all function-level `#[allow(clippy::...)]` attributes when moving code
- `MarketCoin` fields must be `pub(crate)` — `tui_data.rs` reads all fields directly
- `ChartData.prices` must be `pub(crate)` — `tui_data.rs` reads it in `fetch_coin_chart`
- `ChartData.market_caps` and `ChartData.total_volumes` are accessed internally by `display_chart` and `export_chart_csv` in `history.rs` (same module), so they stay private

## Verification

- `cargo build` — compiles
- `cargo clippy -- -D warnings` — no warnings
- `cargo test --all-targets` — all 49 unit + 9 integration tests pass (note: dual bin targets cause duplicated test listings, 49 unique)
- `cg price --ids bitcoin` / `cg markets --total 3` — works
- No changes to `main.rs` or `tui.rs` imports
