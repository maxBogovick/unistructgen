use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use unistructgen_macro::generate_struct_from_json;

// Базовый конфиг генерируем из JSON как из источника правды
// (видео: меняем JSON → структуры обновляются автоматически)

generate_struct_from_json! {
    name = "AppConfig",
    json = include_str!("../config/base.json"),
    serde = true,
    optional = false
}

#[derive(Debug, Serialize)]
struct AppConfigReport {
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
    // JSON — базовая конфигурация
    let base_cfg: AppConfig = serde_json::from_str(include_str!("../config/base.json"))?;

    // .env — overrides
    let env = parse_env_file("config/.env")?;
    let merged = merge_config(base_cfg, env)?;

    validate(&merged)?;

    let report = AppConfigReport {
        app_name: merged.app_name,
        log_level: merged.log_level,
        port: merged.port,
        database_url: merged.database_url,
        max_connections: merged.max_connections,
        timeout_ms: merged.timeout_ms,
        enable_metrics: merged.enable_metrics,
        features: merged.features,
    };

    std::fs::write("config_merged.json", serde_json::to_string_pretty(&report)?)?;
    println!("Merged config written to config_merged.json");

    Ok(())
}
