//! `CoinGecko` API client — HTTP requests, response types, and data formatting.

mod client;
mod date;
mod history;
mod markets;
mod price;
mod search;
mod trending;
mod tui_data;

pub use history::run_history;
pub use markets::run_markets;
pub use price::run_price;
pub use search::run_search;
pub use trending::run_trending;
pub use tui_data::{CoinDetail, MarketEntry};
pub use tui_data::{fetch_coin_chart, fetch_coin_detail, fetch_top_coins, fetch_trending_coins};
