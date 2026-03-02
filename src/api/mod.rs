//! `CoinGecko` API client — HTTP requests, response types, and data formatting.

use comfy_table::Color;

mod client;
mod date;
mod history;
mod markets;
mod price;
mod search;
mod trending;
mod tui_data;

pub(crate) const GOLD: Color = Color::Rgb {
    r: 255,
    g: 215,
    b: 0,
};

pub use history::run_history;
pub use markets::run_markets;
pub use price::run_price;
pub use search::run_search;
pub use trending::run_trending;
pub use tui_data::{CoinDetail, MarketEntry};
pub use tui_data::{fetch_coin_chart, fetch_coin_detail, fetch_top_coins, fetch_trending_coins};
