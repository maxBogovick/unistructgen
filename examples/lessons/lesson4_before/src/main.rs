use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    log_level: String,
    port: u16,
    database_url: String,
    max_connections: u32,
    timeout_ms: u64,
    enable_metrics: bool,
    features: Vec<String>,
}

fn parse_env_file(path: &str) -> Result<HashMap<String, String>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path))?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    Ok(map)
}

fn merge_config(base: AppConfig, env: HashMap<String, String>) -> Result<AppConfig> {
    let mut merged = base;

    if let Some(v) = env.get("APP_NAME") {
        merged.app_name = v.to_string();
    }
    if let Some(v) = env.get("LOG_LEVEL") {
        merged.log_level = v.to_string();
    }
    if let Some(v) = env.get("PORT") {
        merged.port = v.parse()?;
    }
    if let Some(v) = env.get("DATABASE_URL") {
        merged.database_url = v.to_string();
    }
    if let Some(v) = env.get("MAX_CONNECTIONS") {
        merged.max_connections = v.parse()?;
    }
    if let Some(v) = env.get("TIMEOUT_MS") {
        merged.timeout_ms = v.parse()?;
    }
    if let Some(v) = env.get("ENABLE_METRICS") {
        merged.enable_metrics = v.parse()?;
    }

    Ok(merged)
}

fn validate(config: &AppConfig) -> Result<()> {
    if config.port == 0 {
        anyhow::bail!("port must be > 0");
    }
    if !config.database_url.starts_with("postgres://") {
        anyhow::bail!("database_url must start with postgres://");
    }
    if config.timeout_ms < 100 {
        anyhow::bail!("timeout_ms must be >= 100");
    }
    Ok(())
}

fn main() -> Result<()> {
    let base = std::fs::read_to_string("config/base.json")?;
    let base_cfg: AppConfig = serde_json::from_str(&base)?;

    let env = parse_env_file("config/.env")?;
    let merged = merge_config(base_cfg, env)?;

    validate(&merged)?;

    let out = serde_json::to_string_pretty(&merged)?;
    std::fs::write("config_merged.json", out)?;
    println!("Merged config written to config_merged.json");

    Ok(())
}
