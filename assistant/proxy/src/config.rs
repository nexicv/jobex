//! Settings from `config.toml` plus secrets from the environment.

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Address to listen on. Behind Caddy this stays on localhost.
    #[serde(default = "d_listen")]
    pub listen: String,
    /// Browser origins allowed to call the proxy.
    pub allowed_origins: Vec<String>,
    /// Folder that holds SYSTEM_PROMPT.md, KNOWLEDGE.md and tools.json.
    pub assistant_dir: PathBuf,
    /// Take the client address from the last X-Forwarded-For entry (set by Caddy).
    #[serde(default = "d_true")]
    pub trust_forwarded_for: bool,
    pub deepseek: DeepSeek,
    #[serde(default)]
    pub limits: Limits,
    pub prices: Prices,
    #[serde(default)]
    pub quiet_hours: QuietHours,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeepSeek {
    #[serde(default = "d_base")]
    pub base_url: String,
    #[serde(default = "d_model")]
    pub model: String,
    #[serde(default = "d_effort")]
    pub reasoning_effort: String,
    #[serde(default = "d_gate_model")]
    pub gate_model: String,
    #[serde(default = "d_true")]
    pub scope_gate: bool,
    #[serde(default = "d_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "d_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_user_chars: usize,
    pub max_tool_result_chars: usize,
    pub max_messages: usize,
    pub max_tool_rounds: usize,
    pub per_ip_10min: u32,
    pub per_ip_day: u32,
    /// Spending cap in US dollars over any rolling 60 minutes.
    pub usd_per_hour: f64,
    pub team_fail_limit: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_user_chars: 2_000,
            max_tool_result_chars: 12_000,
            max_messages: 40,
            max_tool_rounds: 6,
            // A whole class may share one school IP, so these are generous; the
            // dollar cap is the real brake.
            per_ip_10min: 120,
            per_ip_day: 1_000,
            usd_per_hour: 1.0,
            team_fail_limit: 5,
        }
    }
}

/// US dollars per million tokens, copied from DeepSeek's pricing page.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prices {
    pub input_cache_hit_per_m: f64,
    pub input_cache_miss_per_m: f64,
    pub output_per_m: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuietHours {
    pub enabled: bool,
    /// Weekday (Mon-Fri) windows in UTC, as "HH:MM-HH:MM".
    pub weekday_windows_utc: Vec<String>,
}

impl Default for QuietHours {
    fn default() -> Self {
        Self { enabled: true, weekday_windows_utc: vec!["01:00-04:00".into(), "06:00-10:00".into()] }
    }
}

fn d_listen() -> String {
    "127.0.0.1:8787".into()
}
fn d_true() -> bool {
    true
}
fn d_base() -> String {
    "https://api.deepseek.com".into()
}
fn d_model() -> String {
    "deepseek-flash".into()
}
fn d_effort() -> String {
    "high".into()
}
fn d_gate_model() -> String {
    "deepseek-flash".into()
}
fn d_max_tokens() -> u32 {
    4_000
}
fn d_timeout() -> u64 {
    90
}

/// Secrets, read only from the environment (systemd EnvironmentFile on the server).
pub struct Secrets {
    pub api_key: String,
    pub seal_key: [u8; 32],
    /// Argon2id PHC string, or None when team access is not set up.
    pub team_pass_hash: Option<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Config = toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        let p = &self.prices;
        if !(p.input_cache_hit_per_m > 0.0 && p.input_cache_miss_per_m > 0.0 && p.output_per_m > 0.0) {
            bail!("[prices] must be filled from DeepSeek's pricing page (all three above 0); the spending cap depends on them");
        }
        if self.limits.usd_per_hour <= 0.0 {
            bail!("limits.usd_per_hour must be above 0");
        }
        if self.allowed_origins.is_empty() {
            bail!("allowed_origins is empty");
        }
        crate::quiet::parse_windows(&self.quiet_hours.weekday_windows_utc)?;
        Ok(())
    }
}

impl Secrets {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY").context("DEEPSEEK_API_KEY is not set")?;
        if api_key.trim().is_empty() {
            bail!("DEEPSEEK_API_KEY is empty");
        }
        let seal_hex = std::env::var("SEAL_KEY").context("SEAL_KEY is not set (create one with `jobex-assistant seal-key`)")?;
        let seal_key = parse_hex32(seal_hex.trim()).context("SEAL_KEY must be 64 hex characters")?;
        let team_pass_hash = match std::env::var("TEAM_PASS_HASH") {
            Ok(v) if !v.trim().is_empty() => Some(crate::team::decode_stored_hash(v.trim())?),
            _ => None,
        };
        Ok(Self { api_key: api_key.trim().to_string(), seal_key, team_pass_hash })
    }
}

pub fn parse_hex32(s: &str) -> Result<[u8; 32]> {
    if s.len() != 64 {
        bail!("expected 64 hex characters, got {}", s.len());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let pair = std::str::from_utf8(chunk)?;
        out[i] = u8::from_str_radix(pair, 16).context("not hex")?;
    }
    Ok(out)
}
