# CoinGecko CLI — Rust Edition

A blazingly fast, high-fidelity terminal interface for the [CoinGecko API](https://docs.coingecko.com). Re-engineered from the ground up in **Rust** for near-zero latency, type-safety, and a premium terminal experience — with a full interactive TUI built on `ratatui`.

---

## Why Rust?

- **Near-zero latency** — executes instantly with no runtime overhead (no Node.js, no Python, no JVM)
- **Single binary** — `cargo install` produces one standalone file you can put anywhere
- **Memory safe** — Rust's strict ownership model means no crashes on malformed API responses
- **Type-safe API** — every CoinGecko endpoint is modelled as strict Rust structs with `Option<T>` fallbacks

---

## Installation

You need the Rust toolchain. Get it at [rustup.rs](https://rustup.rs/) if you don't have it.

```bash
git clone https://github.com/sachiew/coingecko-rs.git
cd coingecko-rs
cargo install --path .
```

This installs two aliases: `cg` (short form) and `coingecko` (long form). Both are identical.

---

## Setup

Get a free API key at [coingecko.com/en/api](https://www.coingecko.com/en/api), then run:

```bash
cg auth
```

The interactive prompt walks you through selecting your plan tier (`demo` or `pro`) and saving your key. Credentials are stored securely in your OS config directory (`~/.config/coingecko-cli/` on macOS/Linux).

You can also pass flags directly:

```bash
cg auth --key YOUR_KEY --tier demo
```

Check your saved config at any time:

```bash
cg status
```

---

## Commands

### `cg price` — Live Coin Prices

Fetch the current price of one or more coins. Supports both CoinGecko IDs and ticker symbols. Multiple quote currencies are supported.

```bash
# By ID
cg price --ids bitcoin
cg price --ids bitcoin,ethereum,solana

# By ticker symbol (resolved automatically via search)
cg price --symbols btc
cg price --symbols btc,eth,sol,uni

# Multiple quote currencies
cg price --ids bitcoin --vs usd,eur,gbp

# Mix IDs and symbols
cg price --ids bitcoin --symbols eth,sol
```

**Output columns:** Coin | Price (CURRENCY) | 24h Change — one pair per `--vs` currency.

---

### `cg markets` — Top Coins by Market Cap

Fetch ranked market data with auto-pagination. The API is always queried in 250-coin pages regardless of the total you request, so `--total 1000` makes exactly 4 API calls.

```bash
cg markets
cg markets --total 100
cg markets --total 500 --vs eur
cg markets --total 250 --order gecko_desc
cg markets --total 250 --export data.csv
```

| Flag | Default | Description |
|---|---|---|
| `--total` | `100` | Number of coins to fetch |
| `--vs` | `usd` | Quote currency |
| `--order` | `market_cap_desc` | Sort order (e.g. `volume_desc`, `gecko_desc`) |
| `--export` | — | Export to CSV file path |
| `--category` | — | Filter by category slug (e.g. `layer-2`, `defi`, `tokenized-gold`) |

**Output columns:** # | Name | Symbol | Price | Market Cap | Volume | 24h

---

### `cg search` — Search Coins

Search for any coin by name or symbol. Returns the top matches with their CoinGecko IDs (useful for other commands).

```bash
cg search ethereum
cg search uni --limit 5
```

| Flag | Default | Description |
|---|---|---|
| `--limit` | `10` | Max results to show |

**Output columns:** Rank | Name | Symbol | ID

---

### `cg trending` — Trending (24h)

Shows the three trending tables in one view:

```bash
cg trending
```

- **Top 15 trending coins** — with price and 24h % change
- **Top 7 trending NFTs** — with floor price and 24h % change
- **Top 6 trending categories** — with market cap and 24h % change

---

### `cg history` — Historical Price Data

Three modes for querying historical data. All modes support `--vs` currency and `--export`.

**Single date snapshot:**
```bash
cg history bitcoin --date 2024-01-15
cg history ethereum --date 2024-06-01 --vs eur
```

**Past N days (rolling window):**
```bash
cg history bitcoin --days 7
cg history bitcoin --days 30 --export btc_30d.csv
cg history ethereum --days 90 --vs eur
```

**Custom date range:**
```bash
cg history bitcoin --from 2024-01-01 --to 2024-01-31
cg history bitcoin --from 2024-01-01 --to 2024-03-31 --export q1.csv
```

| Flag | Description |
|---|---|
| `--date YYYY-MM-DD` | Single-day snapshot (price, market cap, volume) |
| `--days N` | Past N days of daily OHLCV data |
| `--from / --to YYYY-MM-DD` | Inclusive date range |
| `--vs` | Quote currency (default: `usd`) |
| `--export` | Export to CSV file path |

---

## CSV Export

`markets` and `history` commands can export raw data to CSV for analysis in Excel, Python, etc.

```bash
# Markets
cg markets --total 500 --export top500.csv

# History
cg history bitcoin --days 30 --export btc_30d.csv
cg history bitcoin --from 2024-01-01 --to 2024-12-31 --export btc_2024.csv
cg history bitcoin --date 2024-01-15 --export btc_snapshot.csv
```

CSV files contain **raw numbers** (not formatted strings), making them directly usable in data pipelines.

---

## 🏷️ Category Filtering (Stocks, Gold, & AI)

Did you know the CoinGecko API tracks Real World Assets (RWAs), commodities, and tokenized stocks? You can filter both the standard markets table and the interactive TUI using the `--category` flag!

**Examples of what you can track:**

```bash
cg markets --category tokenized-gold              # Gold & Silver pegged assets
cg tui --category real-world-assets-rwa           # Real Estate & T-Bills
cg tui --category tokenized-stock                 # Equities & Stocks
cg markets --category artificial-intelligence --total 50   # Top AI coins
cg tui --category solana-meme                     # Solana ecosystem tokens
cg markets --category layer-2 --export l2.csv     # Export all L2 tokens to CSV
```

The `--category` flag works identically in both commands:

- In `cg markets` — filters every page of the pagination loop, so `--total 300 --category defi` correctly returns 300 DeFi coins across multiple API pages
- In `cg tui` — scopes the list view and shows the active category in **gold** in the header bar on every screen (list, loading, and detail)

> **Tip:** Find category slugs by browsing the [CoinGecko categories page](https://www.coingecko.com/en/categories) and copying the slug from the URL (e.g. `coingecko.com/en/categories/layer-2` → `layer-2`).

---

## Interactive TUI

Two full-screen interactive modes built with [`ratatui`](https://ratatui.rs).

### `cg tui` — Top 50 Markets

```bash
cg tui
cg tui --category layer-2
cg tui --category defi
```

Launches a live interactive table of the **top 50 coins by market cap**. Add `--category` to scope the list to any CoinGecko category — the active category is shown in gold in the header on every screen.

**List view columns:** # | Name | Symbol | Price (USD) | Market Cap | Volume | 24h

### `cg tui-trending` — Top 30 Trending

```bash
cg tui-trending
```

Launches a live interactive table of the **top 30 trending coins (24h)**.

**List view columns:** Trend | MCap # | Name | Symbol | Price (USD) | 24h

### Keyboard Controls

| Key | Action |
|---|---|
| `↑` / `k` | Move selection up |
| `↓` / `j` | Move selection down |
| `Enter` | Open detail view |
| `Esc` / `q` / `Backspace` | Back to list (from detail) / Quit (from list) |

### Detail View

Pressing `Enter` on any coin fetches and displays a **split-panel detail view**:

```
┌─ ◆ CoinGecko  Bitcoin (BTC) — Detail ──────────────────────────────────────┐
│ ┌── Info ───────────┐ ┌── 7-Day Price (USD) ──────────────────────────────┐ │
│ │ Rank    1         │ │                                         ▲▲▲▲▲     │ │
│ │ Name    Bitcoin   │ │              ▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲     ▲▲▲ │ │
│ │ Symbol  BTC       │ │         ▲▲▲▲▲                                     │ │
│ │ ID      bitcoin   │ │    ▲▲▲▲▲                                           │ │
│ │                   │ │                                                     │ │
│ │ Price   $95,200   │ └─────────────────────────────────────────────────────┘ │
│ │ Mkt Cap $1.88T    │                                                     │
│ │ Vol 24h $42.10B   │                                                     │
│ │ 24h Chg ▲ 2.34%   │                                                     │
│ │                   │                                                     │
│ │ Hi 24h  $96,100   │                                                     │
│ │ Lo 24h  $93,800   │                                                     │
│ │                   │                                                     │
│ │ ATH     $108,786  │                                                     │
│ │  date   2025-01-20│                                                     │
│ │  from ATH ▼ 12.4% │                                                     │
│ │ ATL     $67.81    │                                                     │
│ │  date   2013-07-06│                                                     │
│ │  from ATL ▲ ...%  │                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Left panel (30%)** — Info:
- Rank, Name, Symbol, CoinGecko ID
- Current price, market cap, 24h volume, 24h change
- 24h high / 24h low
- All-time high: price, date, % from ATH
- All-time low: price, date, % from ATL

**Right panel (70%)** — 7-day price chart:
- Braille-dot line chart rendered in the terminal
- Chart colour matches the 24h trend (green = up, red = down)
- Y-axis shows low / mid / high price labels

Both panels are fetched **concurrently** when you press Enter, so the loading time is the slower of the two requests (not their sum).

---

## Full Command Reference

```
cg [COMMAND]

Commands:
  auth          Save your CoinGecko API key and tier (demo/pro)
  status        Show current auth configuration
  price         Get the current price of one or more coins
  markets       List top coins by market cap
  search        Search for coins, exchanges, and categories
  trending      Show trending coins, NFTs, and categories (24h)
  history       Get historical price data for a coin
  tui           Browse top 50 coins interactively (TUI mode)
  tui-trending  Browse top 30 trending coins interactively (TUI mode)
  help          Print help for a command

Options:
  -h, --help     Print help
  -V, --version  Print version
```

For per-command help:

```bash
cg price --help
cg markets --help
cg history --help
```

---

## Tech Stack

| Crate | Purpose |
|---|---|
| [`clap`](https://docs.rs/clap) | CLI argument parsing (derive API) |
| [`tokio`](https://tokio.rs) | Async runtime |
| [`reqwest`](https://docs.rs/reqwest) | HTTP client |
| [`serde` / `serde_json`](https://serde.rs) | JSON deserialization |
| [`ratatui`](https://ratatui.rs) | Full-screen TUI framework |
| [`crossterm`](https://docs.rs/crossterm) | Cross-platform terminal control |
| [`comfy-table`](https://docs.rs/comfy-table) | Aligned terminal tables |
| [`colored`](https://docs.rs/colored) | Truecolor terminal output |
| [`csv`](https://docs.rs/csv) | CSV export |
| [`directories`](https://docs.rs/directories) | OS config directory |
| [`dialoguer`](https://docs.rs/dialoguer) | Interactive auth prompts |

---

## Roadmap

- [x] `price` — live prices by ID or symbol, multi-currency
- [x] `markets` — paginated top-N by market cap, CSV export
- [x] `search` — coin search
- [x] `trending` — coins, NFTs, categories
- [x] `history` — single date, rolling days, date range, CSV export
- [x] `tui` — interactive top-50 markets with drill-down + 7-day chart
- [x] `tui-trending` — interactive top-30 trending with drill-down + 7-day chart
- [x] `--category` filtering for `markets` and `tui` (DeFi, RWA, tokenized stocks, AI, and more)
- [ ] v1.4.0: Ethereum Gas Tracker
- [ ] v1.5.0: Portfolio tracker (local watchlist)

---

## License

MIT
