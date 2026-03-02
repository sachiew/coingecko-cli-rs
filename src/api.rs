//! `CoinGecko` API client — HTTP requests, response types, and data formatting.

use std::collections::HashMap;

use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::get_credentials;
use crate::ui::{dim, format_change, format_large_usd, format_usd, green_bold};

// ─── HTTP Client ──────────────────────────────────────────────────────────────

struct Client {
    http: reqwest::Client,
    base_url: &'static str,
    header_name: &'static str,
    api_key: Option<String>,
}

impl Client {
    fn build() -> Self {
        let creds = get_credentials();
        Client {
            http: reqwest::Client::builder()
                .user_agent(concat!("coingecko-cli/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("Failed to build HTTP client"),
            base_url: creds.tier.base_url(),
            header_name: creds.tier.header_key(),
            api_key: creds.api_key,
        }
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let req = self.http.get(url);
        match &self.api_key {
            Some(key) => req.header(self.header_name, key),
            None => req,
        }
    }
}

// ─── Price ────────────────────────────────────────────────────────────────────

pub async fn run_price(
    ids: Option<&str>,
    symbols: Option<&str>,
    vs: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build();

    // Resolve each symbol to a CoinGecko ID via /search.
    let mut resolved: Vec<String> = Vec::new();

    if let Some(syms) = symbols {
        for sym in syms.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let path = format!("/search?query={sym}");
            let resp = client.get(&path).send().await?;
            if resp.status().is_success() {
                let body: Value = resp.json().await?;
                if let Some(id) = body["coins"][0]["id"].as_str() {
                    resolved.push(id.to_string());
                } else {
                    eprintln!(
                        "{}",
                        dim(&format!("  ⚠  No match for symbol '{sym}' — skipping.\n"))
                    );
                }
            }
        }
    }

    if let Some(i) = ids {
        for id in i.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            resolved.push(id.to_string());
        }
    }

    // Default to bitcoin when neither flag is given.
    if resolved.is_empty() {
        resolved.push("bitcoin".to_string());
    }

    let ids_param = resolved.join(",");
    let path = format!("/simple/price?ids={ids_param}&vs_currencies={vs}&include_24hr_change=true");

    let resp = client.get(&path).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}").into());
    }

    let data: HashMap<String, HashMap<String, Value>> = resp.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    if data.is_empty() {
        eprintln!(
            "{}",
            dim("  No results — check the coin IDs and try again.\n")
        );
        return Ok(());
    }

    // Support multiple quote currencies (e.g. --vs usd,eur).
    let currencies: Vec<&str> = vs.split(',').map(str::trim).collect();

    let mut entries: Vec<_> = data.iter().collect();
    entries.sort_by_key(|(id, _)| id.as_str());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);

    // Dynamic headers: one Price + one 24h column per currency.
    let gold = Color::Rgb {
        r: 255,
        g: 215,
        b: 0,
    };
    let mut headers = vec![Cell::new("Coin").add_attribute(Attribute::Bold).fg(gold)];
    for cur in &currencies {
        headers.push(
            Cell::new(format!("Price ({})", cur.to_uppercase()))
                .add_attribute(Attribute::Bold)
                .fg(gold),
        );
        headers.push(
            Cell::new(format!("24h ({})", cur.to_uppercase()))
                .add_attribute(Attribute::Bold)
                .fg(gold),
        );
    }
    table.set_header(headers);

    for (coin_id, prices) in entries {
        let mut row = vec![Cell::new(coin_id.as_str())];
        for cur in &currencies {
            let price = prices.get(*cur).and_then(Value::as_f64).unwrap_or(0.0);
            let change_key = format!("{cur}_24h_change");
            let change = prices.get(&change_key).and_then(Value::as_f64);
            row.push(Cell::new(format_usd(price)));
            row.push(match change {
                Some(c) if c >= 0.0 => Cell::new(format!("▲ {:.2}%", c.abs())).fg(Color::Green),
                Some(c) => Cell::new(format!("▼ {:.2}%", c.abs())).fg(Color::Red),
                None => Cell::new("—"),
            });
        }
        table.add_row(row);
    }

    println!("{table}\n");
    Ok(())
}

// ─── Trending ─────────────────────────────────────────────────────────────────
// Deserialized as raw Value to survive any API field-type changes without crashing.

fn change_cell(pct: Option<f64>) -> Cell {
    match pct {
        Some(c) if c >= 0.0 => Cell::new(format!("▲ {:.2}%", c.abs())).fg(Color::Green),
        Some(c) => Cell::new(format!("▼ {:.2}%", c.abs())).fg(Color::Red),
        None => Cell::new("—"),
    }
}

#[allow(clippy::too_many_lines)] // renders 3 tables sequentially; splitting would hurt readability
pub async fn run_trending(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build();
    let resp = client.get("/search/trending").send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}").into());
    }

    // Parse as generic Value — never fails on unexpected field types or names.
    let root: Value = resp.json().await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(());
    }

    let empty = vec![];

    // ── Table 1: Trending Coins ───────────────────────────────────────────────
    println!("  {}\n", green_bold("Trending Coins (Top 15, 24h)"));

    let coins = root["coins"].as_array().unwrap_or(&empty);
    if coins.is_empty() {
        println!("{}\n", dim("  No trending coins found."));
    } else {
        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("Rank")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Coin")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Symbol")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Price (USD)")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
        ]);
        for (i, entry) in coins.iter().enumerate() {
            let item = &entry["item"];
            let rank = item["market_cap_rank"]
                .as_u64()
                .map_or_else(|| format!("#{}", i + 1), |n| n.to_string());
            let name = item["name"].as_str().unwrap_or("—");
            let symbol = item["symbol"].as_str().unwrap_or("—").to_uppercase();
            let price = item["data"]["price"]
                .as_f64()
                .map_or_else(|| "—".to_string(), format_usd);
            let pct = item["data"]["price_change_percentage_24h"]["usd"].as_f64();
            table.add_row(vec![
                Cell::new(rank),
                Cell::new(name),
                Cell::new(symbol),
                Cell::new(price),
                change_cell(pct),
            ]);
        }
        println!("{table}\n");
    }

    // ── Table 2: Trending NFTs ────────────────────────────────────────────────
    println!("  {}\n", green_bold("Trending NFTs (Top 7, 24h)"));

    let nfts = root["nfts"].as_array().unwrap_or(&empty);
    if nfts.is_empty() {
        println!("{}\n", dim("  No trending NFTs found."));
    } else {
        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("Name")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Symbol")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Floor Price")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
        ]);
        for nft in nfts {
            let name = nft["name"].as_str().unwrap_or("—");
            let symbol = nft["symbol"].as_str().unwrap_or("—");
            // floor_price_in_native_currency can be f64 or a string depending on tier
            let floor_val = nft["floor_price_in_native_currency"].as_f64().or_else(|| {
                nft["floor_price_in_native_currency"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
            });
            let currency = nft["native_currency_symbol"]
                .as_str()
                .unwrap_or("")
                .to_uppercase();
            let floor = match floor_val {
                Some(p) if !currency.is_empty() => format!("{p:.4} {currency}"),
                Some(p) => format!("{p:.4}"),
                None => nft["data"]["floor_price"]
                    .as_str()
                    .unwrap_or("—")
                    .to_string(),
            };
            let pct = nft["floor_price_24h_percentage_change"]
                .as_f64()
                .or_else(|| {
                    nft["data"]["floor_price_in_usd_24h_percentage_change"]
                        .as_str()
                        .and_then(|s| s.trim_end_matches('%').parse().ok())
                });
            table.add_row(vec![
                Cell::new(name),
                Cell::new(symbol),
                Cell::new(floor),
                change_cell(pct),
            ]);
        }
        println!("{table}\n");
    }

    // ── Table 3: Trending Categories ──────────────────────────────────────────
    println!("  {}\n", green_bold("Trending Categories (Top 6, 24h)"));

    let cats = root["categories"].as_array().unwrap_or(&empty);
    if cats.is_empty() {
        println!("{}\n", dim("  No trending categories found."));
    } else {
        let mut table = Table::new();
        table.load_preset(UTF8_BORDERS_ONLY);
        table.set_header(vec![
            Cell::new("Category")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Coins")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("Market Cap")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(Color::Rgb {
                    r: 255,
                    g: 215,
                    b: 0,
                }),
        ]);
        for cat in cats {
            let name = cat["name"].as_str().unwrap_or("—");
            let coins = cat["coins_count"]
                .as_u64()
                .map_or_else(|| "—".to_string(), |n| n.to_string());
            let mcap = cat["data"]["market_cap"]
                .as_f64()
                .map_or_else(|| "—".to_string(), format_large_usd);
            let pct = cat["data"]["market_cap_change_percentage_24h"]["usd"].as_f64();
            table.add_row(vec![
                Cell::new(name),
                Cell::new(coins),
                Cell::new(mcap),
                change_cell(pct),
            ]);
        }
        println!("{table}\n");
    }

    Ok(())
}

// ─── Date Helpers ─────────────────────────────────────────────────────────────

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(m: u32, y: i32) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn ymd_to_unix(year: i32, month: u32, day: u32) -> i64 {
    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    for m in 1..month {
        days += days_in_month(m, year);
    }
    days += i64::from(day - 1);
    days * 86400
}

fn unix_to_ymd(ts_sec: i64) -> String {
    let mut days = ts_sec / 86400;
    let mut year = 1970i32;
    loop {
        let dy = if is_leap(year) { 366 } else { 365 };
        if days < dy {
            break;
        }
        days -= dy;
        year += 1;
    }
    let mut month = 1u32;
    loop {
        let dm = days_in_month(month, year);
        if days < dm {
            break;
        }
        days -= dm;
        month += 1;
    }
    format!("{}-{:02}-{:02}", year, month, days + 1)
}

fn parse_ymd(s: &str) -> Option<(i32, u32, u32)> {
    let p: Vec<&str> = s.splitn(3, '-').collect();
    if p.len() < 3 {
        return None;
    }
    Some((p[0].parse().ok()?, p[1].parse().ok()?, p[2].parse().ok()?))
}

fn to_api_date(s: &str) -> Option<String> {
    let (y, m, d) = parse_ymd(s)?;
    Some(format!("{d:02}-{m:02}-{y}"))
}

/// Convert a millisecond f64 timestamp (from JSON) to a date string.
fn ms_to_date(ms: f64) -> String {
    #[allow(clippy::cast_possible_truncation)]
    let secs = (ms as i64) / 1000;
    unix_to_ymd(secs)
}

// ─── Markets ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
struct MarketCoin {
    id: String,
    symbol: String,
    name: String,
    market_cap_rank: Option<u32>,
    current_price: Option<f64>,
    market_cap: Option<f64>,
    total_volume: Option<f64>,
    price_change_percentage_24h: Option<f64>,
}

fn export_markets_csv(coins: &[MarketCoin], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record([
        "Rank",
        "ID",
        "Name",
        "Symbol",
        "Price",
        "Market Cap",
        "Volume 24h",
        "24h Change %",
    ])?;
    for c in coins {
        wtr.write_record(&[
            c.market_cap_rank.map(|r| r.to_string()).unwrap_or_default(),
            c.id.clone(),
            c.name.clone(),
            c.symbol.to_uppercase(),
            c.current_price.map(|p| p.to_string()).unwrap_or_default(),
            c.market_cap.map(|m| m.to_string()).unwrap_or_default(),
            c.total_volume.map(|v| v.to_string()).unwrap_or_default(),
            c.price_change_percentage_24h
                .map(|ch| format!("{ch:.4}"))
                .unwrap_or_default(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[allow(clippy::too_many_lines)] // pagination + CSV export + table rendering in one flow
pub async fn run_markets(
    total: u32,
    vs: &str,
    order: &str,
    export: Option<&str>,
    category: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build();
    let mut coins: Vec<MarketCoin> = Vec::new();
    let category_param = category
        .map(|c| format!("&category={c}"))
        .unwrap_or_default();

    if !json && let Some(cat) = category {
        println!("  Filtering by category: {cat}\n");
    }

    // Always request per_page=250 so that the API's page offset is consistent.
    // page=2&per_page=250 → coins 251-500.
    // page=2&per_page=50  → coins  51-100  ← the old bug.
    // After collecting enough we truncate to exactly `total`.
    let mut page = 1u32;
    while coins.len() < total as usize {
        let path = format!(
            "/coins/markets?vs_currency={vs}&order={order}&per_page=250&page={page}&sparkline=false&price_change_percentage=24h{category_param}"
        );
        let resp = client.get(&path).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}").into());
        }
        let batch: Vec<MarketCoin> = resp.json().await?;
        if batch.is_empty() {
            break;
        }
        coins.extend(batch);
        page += 1;
    }
    // Trim any overshoot from the last 250-coin page.
    coins.truncate(total as usize);

    if json {
        if let Some(path) = export {
            export_markets_csv(&coins, path)?;
            eprintln!("  Exported {} coins to {}", coins.len(), path);
        }
        println!("{}", serde_json::to_string_pretty(&coins)?);
        return Ok(());
    }

    if coins.is_empty() {
        eprintln!("{}", dim("  No coins found.\n"));
        return Ok(());
    }

    if let Some(path) = export {
        export_markets_csv(&coins, path)?;
        println!("  Exported {} coins to {}", coins.len(), path);
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec![
        Cell::new("#")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Name")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Symbol")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new(format!("Price ({})", vs.to_uppercase()))
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Market Cap")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Volume")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("24h")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
    ]);

    for c in &coins {
        let rank = c
            .market_cap_rank
            .map_or_else(|| "—".to_string(), |r| r.to_string());
        let price = c.current_price.map_or_else(|| "—".to_string(), format_usd);
        let mcap = c
            .market_cap
            .map_or_else(|| "—".to_string(), format_large_usd);
        let vol = c
            .total_volume
            .map_or_else(|| "—".to_string(), format_large_usd);
        let change_cell = match c.price_change_percentage_24h {
            Some(ch) if ch >= 0.0 => Cell::new(format!("▲ {:.2}%", ch.abs())).fg(Color::Green),
            Some(ch) => Cell::new(format!("▼ {:.2}%", ch.abs())).fg(Color::Red),
            None => Cell::new("—"),
        };
        table.add_row(vec![
            Cell::new(rank),
            Cell::new(c.name.as_str()),
            Cell::new(c.symbol.to_uppercase()),
            Cell::new(price),
            Cell::new(mcap),
            Cell::new(vol),
            change_cell,
        ]);
    }

    println!("{table}\n");
    Ok(())
}

// ─── Search ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
struct SearchResponse {
    coins: Vec<SearchCoin>,
}

#[derive(Deserialize, Serialize)]
struct SearchCoin {
    id: String,
    name: String,
    symbol: String,
    market_cap_rank: Option<u32>,
}

pub async fn run_search(
    query: &str,
    limit: usize,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build();
    let path = format!("/search?query={query}");
    let resp = client.get(&path).send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("API error {status}: {body}").into());
    }

    let mut data: SearchResponse = resp.json().await?;

    if json {
        data.coins.truncate(limit);
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let coins: Vec<&SearchCoin> = data.coins.iter().take(limit).collect();

    if coins.is_empty() {
        eprintln!("{}", dim("  No results found.\n"));
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec![
        Cell::new("Rank")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Name")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Symbol")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("ID")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
    ]);

    for c in coins {
        let rank = c
            .market_cap_rank
            .map_or_else(|| "—".to_string(), |r| r.to_string());
        table.add_row(vec![
            Cell::new(rank),
            Cell::new(c.name.as_str()),
            Cell::new(c.symbol.to_uppercase()),
            Cell::new(c.id.as_str()),
        ]);
    }

    println!("{table}\n");
    Ok(())
}

// ─── History ──────────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize)]
struct HistoryPoint {
    market_data: Option<HistoryMarketData>,
}

#[derive(Deserialize, Serialize)]
struct HistoryMarketData {
    current_price: HashMap<String, f64>,
    market_cap: HashMap<String, f64>,
    total_volume: HashMap<String, f64>,
}

#[derive(Deserialize, Serialize)]
struct ChartData {
    prices: Vec<Vec<f64>>,
    market_caps: Vec<Vec<f64>>,
    total_volumes: Vec<Vec<f64>>,
}

fn export_chart_csv(data: &ChartData, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["Date", "Price", "Market Cap", "Volume"])?;
    for i in 0..data.prices.len() {
        let ts = data.prices[i].first().copied().unwrap_or(0.0);
        let price = data.prices[i].get(1).copied().unwrap_or(0.0);
        let mcap = data
            .market_caps
            .get(i)
            .and_then(|r| r.get(1))
            .copied()
            .unwrap_or(0.0);
        let vol = data
            .total_volumes
            .get(i)
            .and_then(|r| r.get(1))
            .copied()
            .unwrap_or(0.0);
        wtr.write_record(&[
            ms_to_date(ts),
            price.to_string(),
            mcap.to_string(),
            vol.to_string(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

#[allow(clippy::too_many_lines)] // CSV export + table rendering in one flow
fn display_chart(
    data: &ChartData,
    vs: &str,
    export: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if data.prices.is_empty() {
        if json {
            println!("[]");
        } else {
            eprintln!("{}", dim("  No data available.\n"));
        }
        return Ok(());
    }

    if json {
        if let Some(path) = export {
            export_chart_csv(data, path)?;
            eprintln!("  Exported {} rows to {}", data.prices.len(), path);
        }
        println!("{}", serde_json::to_string_pretty(data)?);
        return Ok(());
    }

    let first_price = data
        .prices
        .first()
        .and_then(|p| p.get(1))
        .copied()
        .unwrap_or(0.0);
    let last_price = data
        .prices
        .last()
        .and_then(|p| p.get(1))
        .copied()
        .unwrap_or(0.0);
    let period_change = if first_price == 0.0 {
        0.0
    } else {
        (last_price - first_price) / first_price * 100.0
    };

    if let Some(path) = export {
        export_chart_csv(data, path)?;
        println!("  Exported {} rows to {}", data.prices.len(), path);
        return Ok(());
    }

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec![
        Cell::new("Date")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new(format!("Price ({})", vs.to_uppercase()))
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Market Cap")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
        Cell::new("Volume")
            .add_attribute(Attribute::Bold)
            .fg(Color::Rgb {
                r: 255,
                g: 215,
                b: 0,
            }),
    ]);

    for i in 0..data.prices.len() {
        let ts = data.prices[i].first().copied().unwrap_or(0.0);
        let price = data.prices[i].get(1).copied().unwrap_or(0.0);
        let mcap = data
            .market_caps
            .get(i)
            .and_then(|r| r.get(1))
            .copied()
            .unwrap_or(0.0);
        let vol = data
            .total_volumes
            .get(i)
            .and_then(|r| r.get(1))
            .copied()
            .unwrap_or(0.0);
        table.add_row(vec![
            Cell::new(ms_to_date(ts)),
            Cell::new(format_usd(price)),
            Cell::new(format_large_usd(mcap)),
            Cell::new(format_large_usd(vol)),
        ]);
    }

    println!("{table}\n");
    println!("  Period change: {}\n", format_change(period_change));
    Ok(())
}

// ─── TUI: Public Market Data ──────────────────────────────────────────────────

pub struct MarketEntry {
    pub rank: u32,
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub price: f64,
    pub market_cap: f64,
    pub volume: f64,
    pub change_24h: f64,
    /// Set only for trending coins — their position in the trending list (1-based).
    pub trending_rank: Option<u32>,
}

pub async fn fetch_top_coins(
    n: u32,
    vs: &str,
    category: Option<&str>,
) -> Result<Vec<MarketEntry>, Box<dyn std::error::Error>> {
    let client = Client::build();
    let mut coins: Vec<MarketEntry> = Vec::new();
    let mut page = 1u32;
    let category_param = category
        .map(|c| format!("&category={c}"))
        .unwrap_or_default();

    while coins.len() < n as usize {
        let path = format!(
            "/coins/markets?vs_currency={vs}&order=market_cap_desc&per_page=250&page={page}&sparkline=false&price_change_percentage=24h{category_param}"
        );
        let resp = client.get(&path).send().await?;
        if !resp.status().is_success() {
            return Err(format!("API error {}", resp.status()).into());
        }
        let batch: Vec<MarketCoin> = resp.json().await?;
        if batch.is_empty() {
            break;
        }
        for c in batch {
            coins.push(MarketEntry {
                rank: c.market_cap_rank.unwrap_or(0),
                id: c.id,
                name: c.name,
                symbol: c.symbol,
                price: c.current_price.unwrap_or(0.0),
                market_cap: c.market_cap.unwrap_or(0.0),
                volume: c.total_volume.unwrap_or(0.0),
                change_24h: c.price_change_percentage_24h.unwrap_or(0.0),
                trending_rank: None,
            });
        }
        page += 1;
    }
    coins.truncate(n as usize);
    Ok(coins)
}

// ─── TUI: Coin Detail Data ────────────────────────────────────────────────────

pub struct CoinDetail {
    pub ath: f64,
    pub ath_change_pct: f64,
    pub ath_date: String,
    pub atl: f64,
    pub atl_change_pct: f64,
    pub atl_date: String,
    pub high_24h: f64,
    pub low_24h: f64,
}

#[derive(Deserialize)]
struct CoinDetailRaw {
    market_data: Option<CoinDetailMarketData>,
}

#[derive(Deserialize)]
struct CoinDetailMarketData {
    ath: HashMap<String, f64>,
    ath_change_percentage: HashMap<String, f64>,
    ath_date: HashMap<String, String>,
    atl: HashMap<String, f64>,
    atl_change_percentage: HashMap<String, f64>,
    atl_date: HashMap<String, String>,
    high_24h: HashMap<String, f64>,
    low_24h: HashMap<String, f64>,
}

pub async fn fetch_coin_detail(
    id: &str,
    vs: &str,
) -> Result<CoinDetail, Box<dyn std::error::Error>> {
    let client = Client::build();
    let path = format!(
        "/coins/{id}?localization=false&tickers=false&community_data=false&developer_data=false"
    );
    let resp = client.get(&path).send().await?;
    if !resp.status().is_success() {
        return Err(format!("API error {}", resp.status()).into());
    }
    let raw: CoinDetailRaw = resp.json().await?;
    let md = raw.market_data.ok_or("No market data")?;
    let trim_date = |s: &str| s.get(..10).unwrap_or(s).to_string();
    Ok(CoinDetail {
        ath: md.ath.get(vs).copied().unwrap_or(0.0),
        ath_change_pct: md.ath_change_percentage.get(vs).copied().unwrap_or(0.0),
        ath_date: md
            .ath_date
            .get(vs)
            .map(|s| trim_date(s))
            .unwrap_or_default(),
        atl: md.atl.get(vs).copied().unwrap_or(0.0),
        atl_change_pct: md.atl_change_percentage.get(vs).copied().unwrap_or(0.0),
        atl_date: md
            .atl_date
            .get(vs)
            .map(|s| trim_date(s))
            .unwrap_or_default(),
        high_24h: md.high_24h.get(vs).copied().unwrap_or(0.0),
        low_24h: md.low_24h.get(vs).copied().unwrap_or(0.0),
    })
}

pub async fn fetch_trending_coins() -> Result<Vec<MarketEntry>, Box<dyn std::error::Error>> {
    let client = Client::build();
    let resp = client.get("/search/trending?show_max=coins").send().await?;
    if !resp.status().is_success() {
        return Err(format!("API error {}", resp.status()).into());
    }
    let root: Value = resp.json().await?;
    let empty = vec![];
    let coins = root["coins"].as_array().unwrap_or(&empty);

    let mut result: Vec<MarketEntry> = Vec::new();
    for (i, entry) in coins.iter().enumerate() {
        let item = &entry["item"];
        let id = item["id"].as_str().unwrap_or("").to_string();
        let name = item["name"].as_str().unwrap_or("—").to_string();
        let symbol = item["symbol"].as_str().unwrap_or("—").to_string();
        // rank holds the trending position (1-based); mcap_rank stored separately via the rank field fallback
        let mcap_rank = u32::try_from(item["market_cap_rank"].as_u64().unwrap_or(0)).unwrap_or(0);
        let price = item["data"]["price"].as_f64().unwrap_or(0.0);
        let change_24h = item["data"]["price_change_percentage_24h"]["usd"]
            .as_f64()
            .unwrap_or(0.0);
        result.push(MarketEntry {
            rank: mcap_rank,
            id,
            name,
            symbol,
            price,
            // market_cap and volume come as formatted strings in trending — not parseable as f64.
            // They are filled in for the detail panel via fetch_coin_detail.
            market_cap: 0.0,
            volume: 0.0,
            change_24h,
            trending_rank: u32::try_from(i + 1).ok(),
        });
    }
    Ok(result)
}

pub async fn fetch_coin_chart(
    id: &str,
    days: u32,
    vs: &str,
) -> Result<Vec<(f64, f64)>, Box<dyn std::error::Error>> {
    let client = Client::build();
    let path = format!("/coins/{id}/market_chart?vs_currency={vs}&days={days}");
    let resp = client.get(&path).send().await?;
    if !resp.status().is_success() {
        return Err(format!("API error {}", resp.status()).into());
    }
    let data: ChartData = resp.json().await?;
    let mut x = 0.0_f64;
    let points = data
        .prices
        .iter()
        .map(|p| {
            let point = (x, p.get(1).copied().unwrap_or(0.0));
            x += 1.0;
            point
        })
        .collect();
    Ok(points)
}

#[allow(clippy::too_many_lines, clippy::too_many_arguments)] // three distinct date modes with their own API calls
pub async fn run_history(
    id: &str,
    date: Option<&str>,
    days: Option<u32>,
    from: Option<&str>,
    to: Option<&str>,
    vs: &str,
    export: Option<&str>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build();

    if let Some(d) = date {
        // Case A: single date snapshot
        let api_date = to_api_date(d).ok_or("Invalid date format. Use YYYY-MM-DD.")?;
        let path = format!("/coins/{id}/history?date={api_date}&localization=false");
        let resp = client.get(&path).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}").into());
        }

        let snapshot: HistoryPoint = resp.json().await?;

        if json {
            if let Some(ref md) = snapshot.market_data
                && let Some(path) = export
            {
                let price = md.current_price.get(vs).copied().unwrap_or(0.0);
                let mcap = md.market_cap.get(vs).copied().unwrap_or(0.0);
                let vol = md.total_volume.get(vs).copied().unwrap_or(0.0);
                let mut wtr = csv::Writer::from_path(path)?;
                wtr.write_record(["date", "price", "market_cap", "volume"])?;
                wtr.write_record(&[
                    d.to_string(),
                    price.to_string(),
                    mcap.to_string(),
                    vol.to_string(),
                ])?;
                wtr.flush()?;
                eprintln!("  Exported 1 row to {path}");
            }
            println!("{}", serde_json::to_string_pretty(&snapshot)?);
            return Ok(());
        }

        match snapshot.market_data {
            None => eprintln!("{}", dim("  No market data available for this date.\n")),
            Some(md) => {
                let price = md.current_price.get(vs).copied().unwrap_or(0.0);
                let mcap = md.market_cap.get(vs).copied().unwrap_or(0.0);
                let vol = md.total_volume.get(vs).copied().unwrap_or(0.0);

                let mut table = Table::new();
                table.load_preset(UTF8_BORDERS_ONLY);
                table.set_header(vec![
                    Cell::new("Metric")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Rgb {
                            r: 255,
                            g: 215,
                            b: 0,
                        }),
                    Cell::new("Value")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Rgb {
                            r: 255,
                            g: 215,
                            b: 0,
                        }),
                ]);
                table.add_row(vec![Cell::new("Date"), Cell::new(d)]);
                table.add_row(vec![
                    Cell::new(format!("Price ({})", vs.to_uppercase())),
                    Cell::new(format_usd(price)),
                ]);
                table.add_row(vec![
                    Cell::new("Market Cap"),
                    Cell::new(format_large_usd(mcap)),
                ]);
                table.add_row(vec![
                    Cell::new("Volume (24h)"),
                    Cell::new(format_large_usd(vol)),
                ]);
                println!("{table}\n");

                if let Some(path) = export {
                    let mut wtr = csv::Writer::from_path(path)?;
                    wtr.write_record(["date", "price", "market_cap", "volume"])?;
                    wtr.write_record(&[
                        d.to_string(),
                        price.to_string(),
                        mcap.to_string(),
                        vol.to_string(),
                    ])?;
                    wtr.flush()?;
                    println!("  Exported 1 row to {path}");
                }
            }
        }
    } else if let Some(n) = days {
        // Case B: past N days
        let path = format!("/coins/{id}/market_chart?vs_currency={vs}&days={n}");
        let resp = client.get(&path).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}").into());
        }

        let chart: ChartData = resp.json().await?;
        display_chart(&chart, vs, export, json)?;
    } else if let (Some(f), Some(t)) = (from, to) {
        // Case C: date range
        let (fy, fm, fd) = parse_ymd(f).ok_or("Invalid --from date. Use YYYY-MM-DD.")?;
        let (ty, tm, td) = parse_ymd(t).ok_or("Invalid --to date. Use YYYY-MM-DD.")?;
        let from_unix = ymd_to_unix(fy, fm, fd);
        let to_unix = ymd_to_unix(ty, tm, td) + 86399;
        let path = format!(
            "/coins/{id}/market_chart/range?vs_currency={vs}&from={from_unix}&to={to_unix}"
        );
        let resp = client.get(&path).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("API error {status}: {body}").into());
        }

        let chart: ChartData = resp.json().await?;
        display_chart(&chart, vs, export, json)?;
    } else {
        eprintln!(
            "{}",
            dim(
                "  Usage: cg history <id> [--date YYYY-MM-DD] [--days N] [--from YYYY-MM-DD --to YYYY-MM-DD]\n"
            )
        );
    }

    Ok(())
}
