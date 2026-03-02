//! Persistent configuration — API key and tier storage via OS config directory.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Tier {
    Demo,
    Pro,
}

impl Tier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Tier::Demo => "demo",
            Tier::Pro => "pro",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "demo" => Some(Tier::Demo),
            "pro" => Some(Tier::Pro),
            _ => None,
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            Tier::Demo => "https://api.coingecko.com/api/v3",
            Tier::Pro => "https://pro-api.coingecko.com/api/v3",
        }
    }

    pub fn header_key(&self) -> &'static str {
        match self {
            Tier::Demo => "x-cg-demo-api-key",
            Tier::Pro => "x-cg-pro-api-key",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ConfigFile {
    api_key: Option<String>,
    tier: Option<String>,
}

pub struct Credentials {
    pub api_key: Option<String>,
    pub tier: Tier,
}

// ─── Config Path ──────────────────────────────────────────────────────────────

fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs = ProjectDirs::from("", "", "coingecko-cli")
        .ok_or("Could not determine OS config directory")?;
    let dir = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.json"))
}

// ─── Read / Write ─────────────────────────────────────────────────────────────

fn read_config() -> ConfigFile {
    let Ok(path) = config_path() else {
        return ConfigFile::default();
    };
    if !path.exists() {
        return ConfigFile::default();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_config(cfg: &ConfigFile) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path()?;
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, json)?;
    Ok(())
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub fn get_credentials() -> Credentials {
    let cfg = read_config();
    let tier = cfg
        .tier
        .as_deref()
        .and_then(Tier::from_str)
        .unwrap_or(Tier::Demo);
    Credentials {
        api_key: cfg.api_key,
        tier,
    }
}

pub fn save_credentials(api_key: &str, tier: &Tier) -> Result<(), Box<dyn std::error::Error>> {
    write_config(&ConfigFile {
        api_key: Some(api_key.to_string()),
        tier: Some(tier.as_str().to_string()),
    })
}

/// Mask an API key for display: show first 6 chars, then asterisks.
pub fn mask_key(key: &str) -> String {
    let visible = key.chars().take(6).collect::<String>();
    let hidden = "*".repeat(key.len().saturating_sub(6));
    format!("{visible}{hidden}")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mask_key ────────────────────────────────────────────────────────────

    #[test]
    fn mask_key_normal() {
        assert_eq!(mask_key("ABCDEFGHIJ"), "ABCDEF****");
    }

    #[test]
    fn mask_key_short() {
        assert_eq!(mask_key("AB"), "AB");
    }

    #[test]
    fn mask_key_empty() {
        assert_eq!(mask_key(""), "");
    }

    #[test]
    fn mask_key_exactly_six() {
        assert_eq!(mask_key("123456"), "123456");
    }

    // ── Tier::as_str ────────────────────────────────────────────────────────

    #[test]
    fn tier_as_str_demo() {
        assert_eq!(Tier::Demo.as_str(), "demo");
    }

    #[test]
    fn tier_as_str_pro() {
        assert_eq!(Tier::Pro.as_str(), "pro");
    }

    // ── Tier::from_str ──────────────────────────────────────────────────────

    #[test]
    fn tier_from_str_lowercase() {
        assert_eq!(Tier::from_str("demo"), Some(Tier::Demo));
        assert_eq!(Tier::from_str("pro"), Some(Tier::Pro));
    }

    #[test]
    fn tier_from_str_mixed_case() {
        assert_eq!(Tier::from_str("Demo"), Some(Tier::Demo));
        assert_eq!(Tier::from_str("PRO"), Some(Tier::Pro));
    }

    #[test]
    fn tier_from_str_invalid() {
        assert_eq!(Tier::from_str("invalid"), None);
        assert_eq!(Tier::from_str(""), None);
    }

    // ── Tier::base_url ──────────────────────────────────────────────────────

    #[test]
    fn tier_base_url_demo() {
        assert!(Tier::Demo.base_url().contains("api.coingecko.com"));
    }

    #[test]
    fn tier_base_url_pro() {
        assert!(Tier::Pro.base_url().contains("pro-api"));
    }

    // ── Tier::header_key ────────────────────────────────────────────────────

    #[test]
    fn tier_header_key_demo() {
        assert_eq!(Tier::Demo.header_key(), "x-cg-demo-api-key");
    }

    #[test]
    fn tier_header_key_pro() {
        assert_eq!(Tier::Pro.header_key(), "x-cg-pro-api-key");
    }
}
