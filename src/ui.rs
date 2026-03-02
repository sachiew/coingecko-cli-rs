//! Terminal UI helpers — colors, logo, welcome box, and number formatting.

use colored::Colorize;

// ─── Brand Colors ─────────────────────────────────────────────────────────────
// Green  #8CC351  — used for logo, section headers, highlights
// Yellow #FFD700  — used for table headers, prompts
// These are approximated via colored's truecolor support.

pub fn green(s: &str) -> String {
    s.truecolor(140, 195, 81).to_string()
}

pub fn green_bold(s: &str) -> String {
    s.truecolor(140, 195, 81).bold().to_string()
}

pub fn yellow_bold(s: &str) -> String {
    s.truecolor(255, 215, 0).bold().to_string()
}

pub fn dim(s: &str) -> String {
    s.dimmed().to_string()
}

// ─── ASCII Art Logo ────────────────────────────────────────────────────────────
pub fn print_logo() {
    let logo = [
        "  ██████╗ ██████╗ ██╗███╗   ██╗ ██████╗ ███████╗ ██████╗██╗  ██╗ ██████╗ ",
        " ██╔════╝██╔═══██╗██║████╗  ██║██╔════╝ ██╔════╝██╔════╝██║ ██╔╝██╔═══██╗",
        " ██║     ██║   ██║██║██╔██╗ ██║██║  ███╗█████╗  ██║     █████╔╝ ██║   ██║",
        " ██║     ██║   ██║██║██║╚██╗██║██║   ██║██╔══╝  ██║     ██╔═██╗ ██║   ██║",
        " ╚██████╗╚██████╔╝██║██║ ╚████║╚██████╔╝███████╗╚██████╗██║  ██╗╚██████╔╝",
        "  ╚═════╝ ╚═════╝ ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝ ╚═════╝╚═╝  ╚═╝ ╚═════╝ ",
    ]
    .join("\n");

    println!("\n{}", green(&logo));
}

// ─── Welcome Box ──────────────────────────────────────────────────────────────
// Box outer width = 80 visible chars.
//   Top/bottom : "+" + 78×"-" + "+"          = 80
//   Blank rows : "|" + 78×" " + "|"          = 80
//   Content    : "| " + 76-char content + " |" = 80
//
// ANSI escape codes have byte length but zero terminal-column width, so we
// NEVER use format! width specifiers on colored strings — they count bytes,
// not columns. Instead every helper appends plain-ASCII spaces for padding.

// Safe for plain (uncolored) ASCII text only.
fn plain_row(text: &str) -> String {
    format!("| {text:<76} |")
}

// For rows that contain colored text: pass the string and its true visible
// character count; the helper appends the correct number of spaces.
fn colored_row(content: &str, visible: usize) -> String {
    let pad = 76usize.saturating_sub(visible);
    format!("| {}{} |", content, " ".repeat(pad))
}

// Command row layout (all values are visible-column counts):
//   "| " (2) + "  " (2) + "$" (1) + " " (1) + cmd (≤30) + " " (1)
//   + comment (N) + pad (41-N) + " |" (2)  =  80
fn cmd_row(cmd: &str, comment: &str) -> String {
    let pad = 41usize.saturating_sub(comment.len());
    format!(
        "|   {} {:<30} {}{} |",
        green("$"),
        cmd,
        dim(comment),
        " ".repeat(pad)
    )
}

pub fn print_welcome_box() {
    let top = "+------------------------------------------------------------------------------+";
    let blank = "|                                                                              |";
    let sep = colored_row(&dim(&"-".repeat(76)), 76);

    println!("{top}");
    println!("{blank}");
    // "Official API Command Line Interface" = 35 visible chars; pad = 41
    println!(
        "{}",
        colored_row(&yellow_bold("Official API Command Line Interface"), 35)
    );
    println!("{blank}");
    println!("{sep}");
    println!("{blank}");
    println!("{}", plain_row("  Quick Start"));
    println!("{blank}");
    println!("{}", cmd_row("cg auth", "# Set up your API key"));
    println!("{}", cmd_row("cg price --ids bitcoin", "# Get BTC price"));
    println!(
        "{}",
        cmd_row("cg markets --total 100", "# Top 100 by mkt cap")
    );
    println!("{}", cmd_row("cg search ethereum", "# Search for a coin"));
    println!("{}", cmd_row("cg trending", "# Trending coins"));
    println!(
        "{}",
        cmd_row("cg history bitcoin -d 30", "# 30-day history")
    );
    println!("{blank}");
    println!("{sep}");
    println!("{blank}");
    // "  Docs: https://docs.coingecko.com" → 2 + 6 + 26 = 34 visible chars
    println!(
        "{}",
        colored_row(
            &format!("  {}{}", dim("Docs: "), "https://docs.coingecko.com".cyan()),
            34,
        )
    );
    println!("{blank}");
    println!("{top}");
    println!();
}

// ─── Compact Banner ───────────────────────────────────────────────────────────
pub fn print_banner() {
    println!(
        "\n  {} {}",
        green_bold("◆ CoinGecko"),
        dim("CLI  —  Real-time crypto data\n")
    );
}

// ─── Formatting Helpers ───────────────────────────────────────────────────────

/// Format a float as USD with 2–8 decimal places (matches JS Intl.NumberFormat).
pub fn format_usd(value: f64) -> String {
    // Determine decimal places: use more for small values
    let decimals = if value.abs() >= 1.0 {
        2
    } else if value.abs() >= 0.01 {
        4
    } else if value.abs() >= 0.0001 {
        6
    } else {
        8
    };

    // Build integer part with thousands separators
    let rounded = format!("{value:.decimals$}");
    let parts: Vec<&str> = rounded.splitn(2, '.').collect();
    let int_str = parts[0].trim_start_matches('-');
    let dec_str = if parts.len() > 1 { parts[1] } else { "" };

    let mut int_formatted = String::new();
    for (i, ch) in int_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            int_formatted.push(',');
        }
        int_formatted.push(ch);
    }
    let int_formatted: String = int_formatted.chars().rev().collect();

    let sign = if value < 0.0 { "-" } else { "" };
    format!("${sign}{int_formatted}.{dec_str}")
}

/// Format large USD values with T/B/M suffix.
pub fn format_large_usd(value: f64) -> String {
    if value >= 1e12 {
        format!("${:.2}T", value / 1e12)
    } else if value >= 1e9 {
        format!("${:.2}B", value / 1e9)
    } else if value >= 1e6 {
        format!("${:.2}M", value / 1e6)
    } else {
        format_usd(value)
    }
}

/// Format a percentage change with colored ▲/▼ arrow.
pub fn format_change(value: f64) -> String {
    let fixed = format!("{:.2}%", value.abs());
    if value >= 0.0 {
        format!("▲ {fixed}").green().to_string()
    } else {
        format!("▼ {fixed}").red().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_usd ──────────────────────────────────────────────────────────

    #[test]
    fn format_usd_whole_number() {
        assert_eq!(format_usd(1234.0), "$1,234.00");
    }

    #[test]
    fn format_usd_thousands_separator() {
        assert_eq!(format_usd(1_000_000.5), "$1,000,000.50");
    }

    #[test]
    fn format_usd_small_2_decimal() {
        assert_eq!(format_usd(9.99), "$9.99");
    }

    #[test]
    fn format_usd_small_4_decimal() {
        // 0.01 <= value < 1.0 → 4 decimals
        assert_eq!(format_usd(0.1234), "$0.1234");
    }

    #[test]
    fn format_usd_small_6_decimal() {
        // 0.0001 <= value < 0.01 → 6 decimals
        assert_eq!(format_usd(0.005678), "$0.005678");
    }

    #[test]
    fn format_usd_small_8_decimal() {
        // value < 0.0001 → 8 decimals
        assert_eq!(format_usd(0.00001234), "$0.00001234");
    }

    #[test]
    fn format_usd_zero() {
        // 0.0 abs < 0.0001 → 8 decimal places
        assert_eq!(format_usd(0.0), "$0.00000000");
    }

    #[test]
    fn format_usd_negative() {
        // sign placed after $
        assert_eq!(format_usd(-42.5), "$-42.50");
    }

    // ── format_large_usd ────────────────────────────────────────────────────

    #[test]
    fn format_large_usd_trillions() {
        assert_eq!(format_large_usd(2.5e12), "$2.50T");
    }

    #[test]
    fn format_large_usd_billions() {
        assert_eq!(format_large_usd(1.23e9), "$1.23B");
    }

    #[test]
    fn format_large_usd_millions() {
        assert_eq!(format_large_usd(456.78e6), "$456.78M");
    }

    #[test]
    fn format_large_usd_fallback() {
        // Below 1M falls through to format_usd
        assert_eq!(format_large_usd(999_999.0), "$999,999.00");
    }

    // ── format_change ───────────────────────────────────────────────────────

    #[test]
    fn format_change_positive() {
        let result = format_change(5.25);
        assert!(result.contains("▲"));
        assert!(result.contains("5.25%"));
    }

    #[test]
    fn format_change_negative() {
        let result = format_change(-3.14);
        assert!(result.contains("▼"));
        assert!(result.contains("3.14%"));
    }

    #[test]
    fn format_change_zero() {
        let result = format_change(0.0);
        assert!(result.contains("▲"));
        assert!(result.contains("0.00%"));
    }
}
