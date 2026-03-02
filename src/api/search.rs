use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};

use super::client::Client;
use crate::ui::dim;

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
    let client = Client::build()?;
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
