use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Ручные структуры под GitHub releases/latest
#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "size")]
    size: i64,
    #[serde(rename = "download_count")]
    download_count: i64,
    #[serde(rename = "browser_download_url")]
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    #[serde(rename = "id")]
    id: i64,
    #[serde(rename = "tag_name")]
    tag_name: String,
    #[serde(rename = "name")]
    name: Option<String>,
    #[serde(rename = "published_at")]
    published_at: Option<String>,
    #[serde(rename = "html_url")]
    html_url: Option<String>,
    #[serde(rename = "assets")]
    assets: Option<Vec<ReleaseAsset>>,
}

#[derive(Debug, Serialize)]
struct ReleaseReport {
    repo: String,
    tag: String,
    title: String,
    published_at: String,
    url: String,
    assets_total: usize,
    downloads_total: i64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let owner = std::env::var("GITHUB_OWNER").unwrap_or_else(|_| "rust-lang".to_string());
    let repo = std::env::var("GITHUB_REPO").unwrap_or_else(|_| "rust".to_string());
    let token = std::env::var("GITHUB_TOKEN").ok();

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    let client = reqwest::Client::new();
    let mut req = client
        .get(&url)
        .header("User-Agent", "unistructgen-demo")
        .header("Accept", "application/vnd.github+json");

    if let Some(token) = token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    let release: ReleaseResponse = req
        .send()
        .await
        .context("GitHub API request failed")?
        .json()
        .await
        .context("Failed to parse JSON")?;

    let assets = release.assets.unwrap_or_default();
    let downloads_total = assets.iter().map(|a| a.download_count).sum();

    let report = ReleaseReport {
        repo: format!("{}/{}", owner, repo),
        tag: release.tag_name,
        title: release.name.unwrap_or_else(|| "(no title)".to_string()),
        published_at: release.published_at.unwrap_or_else(|| "unknown".to_string()),
        url: release.html_url.unwrap_or_else(|| url.clone()),
        assets_total: assets.len(),
        downloads_total,
    };

    let output = json!(report);
    std::fs::write("report.json", serde_json::to_string_pretty(&output)?)?;
    println!("Report saved to report.json");

    Ok(())
}
