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

fn config_path() -> PathBuf {
    let dirs = ProjectDirs::from("", "", "coingecko-cli")
        .expect("Could not determine OS config directory");
    let dir = dirs.config_dir().to_path_buf();
    fs::create_dir_all(&dir).expect("Could not create config directory");
    dir.join("config.json")
}

// ─── Read / Write ─────────────────────────────────────────────────────────────

fn read_config() -> ConfigFile {
    let path = config_path();
    if !path.exists() {
        return ConfigFile::default();
    }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    serde_json::from_str(&raw).unwrap_or_default()
}

fn write_config(cfg: &ConfigFile) {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg).expect("Failed to serialize config");
    fs::write(&path, json).expect("Failed to write config file");
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

pub fn save_credentials(api_key: &str, tier: &Tier) {
    write_config(&ConfigFile {
        api_key: Some(api_key.to_string()),
        tier: Some(tier.as_str().to_string()),
    });
}

/// Mask an API key for display: show first 6 chars, then asterisks.
pub fn mask_key(key: &str) -> String {
    let visible = key.chars().take(6).collect::<String>();
    let hidden = "*".repeat(key.len().saturating_sub(6));
    format!("{}{}", visible, hidden)
}
