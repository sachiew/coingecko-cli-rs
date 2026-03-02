use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::client::Client;
use super::history::ChartData;
use super::markets::MarketCoin;

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
    let client = Client::build()?;
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

// ─── Coin Detail Data ────────────────────────────────────────────────────────

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
    let client = Client::build()?;
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
    let client = Client::build()?;
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

#[allow(clippy::cast_precision_loss)] // sequential chart index — always small
pub async fn fetch_coin_chart(
    id: &str,
    days: u32,
    vs: &str,
) -> Result<Vec<(f64, f64)>, Box<dyn std::error::Error>> {
    let client = Client::build()?;
    let path = format!("/coins/{id}/market_chart?vs_currency={vs}&days={days}");
    let resp = client.get(&path).send().await?;
    if !resp.status().is_success() {
        return Err(format!("API error {}", resp.status()).into());
    }
    let data: ChartData = resp.json().await?;
    let points = data
        .prices
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f64, p.get(1).copied().unwrap_or(0.0)))
        .collect();
    Ok(points)
}
