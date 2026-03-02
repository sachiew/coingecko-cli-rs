use std::collections::HashMap;

use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::{Attribute, Cell, Color, Table};
use serde::{Deserialize, Serialize};

use super::client::Client;
use super::date::{ms_to_date, parse_ymd, to_api_date, ymd_to_unix};
use crate::ui::{dim, format_change, format_large_usd, format_usd};

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
pub(crate) struct ChartData {
    pub(crate) prices: Vec<Vec<f64>>,
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
    let client = Client::build()?;

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
