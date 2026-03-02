use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Table};
use serde::{Deserialize, Serialize};

use super::client::Client;
use crate::ui::{dim, format_large_usd, format_usd};

#[derive(Deserialize, Serialize)]
pub(crate) struct MarketCoin {
    pub(crate) id: String,
    pub(crate) symbol: String,
    pub(crate) name: String,
    pub(crate) market_cap_rank: Option<u32>,
    pub(crate) current_price: Option<f64>,
    pub(crate) market_cap: Option<f64>,
    pub(crate) total_volume: Option<f64>,
    pub(crate) price_change_percentage_24h: Option<f64>,
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
    let client = Client::build()?;
    let mut coins: Vec<MarketCoin> = Vec::new();

    if !json && let Some(cat) = category {
        println!("  Filtering by category: {cat}\n");
    }

    // Always request per_page=250 so that the API's page offset is consistent.
    // page=2&per_page=250 → coins 251-500.
    // page=2&per_page=50  → coins  51-100  ← the old bug.
    // After collecting enough we truncate to exactly `total`.
    let mut page = 1u32;
    while coins.len() < total as usize {
        let page_str = page.to_string();
        let mut params = vec![
            ("vs_currency", vs),
            ("order", order),
            ("per_page", "250"),
            ("page", &page_str),
            ("sparkline", "false"),
            ("price_change_percentage", "24h"),
        ];
        if let Some(cat) = category {
            params.push(("category", cat));
        }
        let resp = client.get("/coins/markets").query(&params).send().await?;
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
            .fg(super::GOLD),
        Cell::new("Name")
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
        Cell::new("Symbol")
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
        Cell::new(format!("Price ({})", vs.to_uppercase()))
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
        Cell::new("Market Cap")
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
        Cell::new("Volume")
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
        Cell::new("24h")
            .add_attribute(Attribute::Bold)
            .fg(super::GOLD),
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
        let change_cell = super::trending::change_cell(c.price_change_percentage_24h);
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
