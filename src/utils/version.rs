use anyhow::Result;
use colored::Colorize;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct CrateResponse {
    version: String,
}

pub async fn check_is_latest_version() -> Result<()> {
    // Create a client with a short timeout
    let client = Client::builder().timeout(Duration::from_secs(2)).build()?;

    // Try to fetch the latest version from crates.io
    let url = format!("https://crates.io/api/v1/crates/{CRATE_NAME}");
    let response = match client
        .get(&url)
        .header("User-Agent", format!("{CRATE_NAME}/{CURRENT_VERSION}"))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => {
            // Silently fail if we can't reach crates.io
            return Ok(());
        }
    };

    if !response.status().is_success() {
        return Ok(());
    }

    let crate_info: CrateResponse = match response.json().await {
        Ok(info) => info,
        Err(_) => return Ok(()),
    };

    // Compare versions
    if crate_info.version != CURRENT_VERSION {
        println!(
            "{}",
            format!(
                "📦 A new version of {} is available: {} → {}",
                CRATE_NAME, CURRENT_VERSION, crate_info.version
            )
            .yellow()
        );
        println!(
            "{}",
            format!("Run `cargo install {CRATE_NAME}` to update").yellow()
        );
        println!();
    }

    Ok(())
}
