use std::collections::HashMap;

use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use serde_json::Value;

use super::client::Client;
use crate::ui::{dim, format_usd};

pub async fn run_price(
    ids: Option<&str>,
    symbols: Option<&str>,
    vs: &str,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::build()?;

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
