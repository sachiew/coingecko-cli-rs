//! CLI entry point — argument parsing, command dispatch, and auth/status flows.

mod api;
mod config;
mod tui;
mod ui;

use clap::{Parser, Subcommand};
use colored::Colorize;
use config::{Tier, get_credentials, mask_key, save_credentials};
use dialoguer::{Input, Select};
use ui::{dim, green_bold, print_banner, print_logo, print_welcome_box, yellow_bold};

// ─── CLI Definition ───────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "cg",
    version = env!("CARGO_PKG_VERSION"),
    about = "CoinGecko CLI — real-time crypto data from the terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Save your `CoinGecko` API key and tier (demo/pro)
    Auth {
        /// Your `CoinGecko` API key
        #[arg(long)]
        key: Option<String>,

        /// Your plan tier: demo or pro
        #[arg(long)]
        tier: Option<String>,
    },

    /// Show current auth configuration
    Status,

    // Placeholder stubs — implemented in the next step
    /// Get the current price of one or more coins
    Price {
        /// Coin IDs, comma-separated (e.g. bitcoin,ethereum)
        #[arg(long)]
        ids: Option<String>,
        /// Ticker symbols, comma-separated — resolved to IDs via search (e.g. btc,eth)
        #[arg(long)]
        symbols: Option<String>,
        /// Quote currencies, comma-separated (e.g. usd,eur)
        #[arg(long, default_value = "usd")]
        vs: String,
    },

    /// List top coins by market cap
    Markets {
        #[arg(long, default_value = "100")]
        total: u32,
        #[arg(long, default_value = "usd")]
        vs: String,
        #[arg(long, default_value = "market_cap_desc")]
        order: String,
        #[arg(long)]
        export: Option<String>,
        /// Filter by `CoinGecko` category id (e.g. layer-2, decentralized-finance-defi, non-fungible-tokens-nft)
        #[arg(long)]
        category: Option<String>,
    },

    /// Search for coins, exchanges, and categories
    Search {
        query: String,
        #[arg(long, default_value = "10")]
        limit: usize,
    },

    /// Show trending coins, NFTs, and categories (24h)
    Trending,

    /// Browse top 50 coins interactively (TUI mode)
    Tui {
        /// Filter by `CoinGecko` category id (e.g. layer-2, decentralized-finance-defi, non-fungible-tokens-nft)
        #[arg(long)]
        category: Option<String>,
    },

    /// Browse top 30 trending coins interactively (TUI mode)
    #[command(name = "tui-trending")]
    TuiTrending,

    /// Get historical price data for a coin
    History {
        id: String,
        #[arg(long)]
        date: Option<String>,
        #[arg(long)]
        days: Option<u32>,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        to: Option<String>,
        #[arg(long, default_value = "usd")]
        vs: String,
        #[arg(long)]
        export: Option<String>,
    },
}

// ─── Entry Point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // Bare `cg` — show branded landing screen
            print_logo();
            print_welcome_box();
        }

        Some(Commands::Auth { key, tier }) => {
            run_auth(key, tier);
        }

        Some(Commands::Status) => {
            run_status();
        }

        Some(Commands::Price { ids, symbols, vs }) => {
            print_banner();
            if let Err(e) = api::run_price(ids.as_deref(), symbols.as_deref(), &vs).await {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::Trending) => {
            print_banner();
            if let Err(e) = api::run_trending().await {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::Tui { category }) => {
            if let Err(e) = tui::run_tui(category.as_deref()).await {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::TuiTrending) => {
            if let Err(e) = tui::run_trending_tui().await {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::Markets {
            total,
            vs,
            order,
            export,
            category,
        }) => {
            print_banner();
            if let Err(e) =
                api::run_markets(total, &vs, &order, export.as_deref(), category.as_deref()).await
            {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::Search { query, limit }) => {
            print_banner();
            if let Err(e) = api::run_search(&query, limit).await {
                eprintln!("  ✖  {e}");
            }
        }

        Some(Commands::History {
            id,
            date,
            days,
            from,
            to,
            vs,
            export,
        }) => {
            print_banner();
            if let Err(e) = api::run_history(
                &id,
                date.as_deref(),
                days,
                from.as_deref(),
                to.as_deref(),
                &vs,
                export.as_deref(),
            )
            .await
            {
                eprintln!("  ✖  {e}");
            }
        }
    }
}

// ─── Auth Command ─────────────────────────────────────────────────────────────

fn run_auth(key_flag: Option<String>, tier_flag: Option<String>) {
    print_banner();

    // ── Tier selection ────────────────────────────────────────────────────────
    let tier = if let Some(t) = tier_flag {
        if let Some(tier) = Tier::from_str(&t) {
            tier
        } else {
            eprintln!("{}", "  ✖  Tier must be \"demo\" or \"pro\"".red());
            std::process::exit(1);
        }
    } else {
        let choices = &[
            "demo  — Free tier (public API)",
            "pro   — Paid tier (pro API)",
        ];
        let idx = Select::new()
            .with_prompt(yellow_bold("  Select your API tier"))
            .items(choices)
            .default(0)
            .interact()
            .unwrap_or_else(|_| {
                eprintln!("{}", "  ✖  Invalid selection".red());
                std::process::exit(1);
            });
        if idx == 0 { Tier::Demo } else { Tier::Pro }
    };

    // ── API Key ───────────────────────────────────────────────────────────────
    let api_key = if let Some(k) = key_flag {
        if k.is_empty() {
            eprintln!("{}", "\n  ✖  API key cannot be empty\n".red());
            std::process::exit(1);
        }
        k
    } else {
        println!(
            "\n  {}{}\n",
            dim("Get your free key at: "),
            "https://www.coingecko.com/en/api".cyan()
        );
        let k: String = Input::new()
            .with_prompt(yellow_bold("  Enter your API key"))
            .interact_text()
            .unwrap_or_else(|_| {
                eprintln!("{}", "\n  ✖  Cancelled\n".red());
                std::process::exit(1);
            });
        if k.is_empty() {
            eprintln!("{}", "\n  ✖  API key cannot be empty\n".red());
            std::process::exit(1);
        }
        k
    };

    save_credentials(&api_key, &tier);

    let masked = mask_key(&api_key);

    println!();
    println!("  ╭──────────────────────────────────╮");
    println!("  │  {}       │", green_bold("✔  Credentials saved"));
    println!("  │                                  │");
    println!(
        "  │  {}  {:<24}│",
        "Tier".truecolor(255, 215, 0).bold(),
        tier.as_str()
    );
    println!(
        "  │  {}  {:<24}│",
        "Key ".truecolor(255, 215, 0).bold(),
        masked
    );
    println!("  ╰──────────────────────────────────╯");
    println!();
}

// ─── Status Command ───────────────────────────────────────────────────────────

fn run_status() {
    print_banner();
    let creds = get_credentials();

    match creds.api_key {
        None => {
            println!("{}", "  ⚠  No credentials configured.".red());
            println!("{}", dim("     Run: cg auth\n"));
        }
        Some(key) => {
            let masked = mask_key(&key);
            println!("{}", green_bold("  ✔  Credentials configured"));
            println!(
                "     {}  {}",
                "Tier".truecolor(255, 215, 0).bold(),
                creds.tier.as_str()
            );
            println!(
                "     {}  {}\n",
                "Key ".truecolor(255, 215, 0).bold(),
                masked
            );
        }
    }
}
