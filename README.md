# CoinGecko CLI — Rust Edition

A blazingly fast, high-fidelity terminal interface for the [CoinGecko API](https://docs.coingecko.com). Re-engineered from the ground up in **Rust** for near-zero latency, type-safety, and a premium terminal experience — with a full interactive TUI built on `ratatui`.

> [!NOTE]
> CoinGecko CLI is currently in Beta.
> We’re constantly improving, and your feedback is crucial. Please share your feedback via this [form](https://forms.gle/VgpVbwsSJLgE7D8Q7), or submit a PR.


## 🌟 Features at a Glance
- 🎮 Interactive TUI Dashboard: A high-fidelity terminal interface with live-navigation and 7-day price charts.
- ⚡ Real-Time Prices: Blazingly fast, type-safe API calls for the most current market valuations.
- 📅 Deep Historical Data: Fetch precise data for specific dates, custom date ranges, or the past N days.
- 📥 CSV Export Support: Export any market or history query directly to CSV for external analysis in Excel or Python.
- 🏷️ Category Smart: Filter by over 500+ categories including AI, Layer-2, Tokenized Stocks, Gold, and Silver.
- 📊 Unlimited Markets: Seamless pagination to fetch 1,000+ coins in a single command.
- 🔥 Trending Everything: Real-time tracking of Trending Coins, NFTs, and Categories.

<img width="609" height="670" alt="Screenshot 2026-03-01 at 7 36 43 AM" src="https://github.com/user-attachments/assets/0fe7e997-35cf-4064-9275-d68eeeccf600" />

<img width="575" height="346" alt="Screenshot 2026-03-01 at 7 12 39 AM" src="https://github.com/user-attachments/assets/41b0c5f6-03b0-418a-af93-6f449e3c4002" />
<img width="573" height="350" alt="Screenshot 2026-03-01 at 7 12 50 AM" src="https://github.com/user-attachments/assets/d13ba5c5-0e9a-4e0e-850c-51528e277828" />


---

## Why Rust?

- **Near-zero latency** — executes instantly with no runtime overhead (no Node.js, no Python, no JVM)
- **Single binary** — `cargo install` produces one standalone file you can put anywhere
- **Memory safe** — Rust's strict ownership model means no crashes on malformed API responses
- **Type-safe API** — every CoinGecko endpoint is modelled as strict Rust structs with `Option<T>` fallbacks

---

## 📦 Installation

This tool is built in Rust. To install it, you must have the [Rust toolchain](https://rustup.rs/) installed on your system.

### Install via Cargo
You can install `coingecko-cli-rs` directly from the source without manual cloning:

```bash
cargo install --git https://github.com/sachiew/coingecko-cli-rs.git
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

> **Tip:** Find coin ids by browsing the respective [CoinGecko coin page](https://www.coingecko.com/en/categories](https://www.coingecko.com/en/coins/bitcoin)) and copying the 'API ID'. You can also get the full list of coin ids via this [endpoint](https://docs.coingecko.com/reference/coins-list) or [google sheet](https://docs.google.com/spreadsheets/d/1wTTuxXt8n9q7C4NDXqQpI3wpKu1_5bGVmP9Xz0XGSyU/edit?gid=0#gid=0).

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
| `--category` | — | Filter by category id (e.g. `layer-2`, `decentralized-finance-defi`, `tokenized-gold`) |

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

- In `cg markets` — filters every page of the pagination loop, so `--total 300 --category decentralized-finance-defi` correctly returns 300 DeFi coins across multiple API pages
- In `cg tui` — scopes the list view and shows the active category in **gold** in the header bar on every screen (list, loading, and detail)

> **Tip:** Find category ids by browsing the [CoinGecko categories page](https://www.coingecko.com/en/categories) and copying the id of respective category page. You can also get the full list of category ids via this [endpoint](https://docs.coingecko.com/reference/coins-categories-list) or [google sheet](https://docs.google.com/spreadsheets/d/1wTTuxXt8n9q7C4NDXqQpI3wpKu1_5bGVmP9Xz0XGSyU/edit?gid=214581757#gid=214581757).

---

## Interactive TUI

Two full-screen interactive modes built with [`ratatui`](https://ratatui.rs).

### `cg tui` — Top 50 Markets

```bash
cg tui
cg tui --category layer-2
cg tui --category decentralized-finance-defi
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

<img width="692" height="411" alt="Screenshot 2026-03-01 at 6 56 23 AM" src="https://github.com/user-attachments/assets/6e7639ba-26d1-499e-8a6d-fc8a3664d88a" />


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

## License

MIT
