use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use serde_json::Value;

use super::client::Client;
use crate::ui::{dim, format_large_usd, format_usd, green_bold};

pub(crate) fn change_cell(pct: Option<f64>) -> Cell {
    match pct {
        Some(c) if c >= 0.0 => Cell::new(format!("▲ {:.2}%", c.abs())).fg(Color::Green),
        Some(c) => Cell::new(format!("▼ {:.2}%", c.abs())).fg(Color::Red),
        None => Cell::new("—"),
    }
}

#[allow(clippy::too_many_lines)] // renders 3 tables sequentially; splitting would hurt readability
pub async fn run_trending(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build()?;
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
                .fg(super::GOLD),
            Cell::new("Coin")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("Symbol")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("Price (USD)")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
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
                .fg(super::GOLD),
            Cell::new("Symbol")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("Floor Price")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
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
                .fg(super::GOLD),
            Cell::new("Coins")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("Market Cap")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
            Cell::new("24h Change")
                .add_attribute(Attribute::Bold)
                .fg(super::GOLD),
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
