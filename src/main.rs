//! A bot which posts the weather forcast for tomorrow

use anyhow::{bail, Result};
use chrono::Local;
use elefren::{status_builder::Visibility, Language, Mastodon, MastodonClient, StatusBuilder};
use tracing::{debug, info};

mod config;
mod vader;
use config::Config;
use vader::Wttr;

// TODO: Make location configurable
async fn fetch_tomorrows_weather(location: &str) -> Result<String> {
    reqwest::Client::new()
        .get(format!("https://wttr.in/{}?format=j1", location))
        .header("Accept-Language", "sv-SE")
        .send()
        .await?
        .json::<Wttr>()
        .await
        .map_err(|e| e.into())
        .map(|wttr| {
            debug!("Fetched weather for: {:?}", wttr.weather[1].date);
            wttr
        })
        .map(|wttr| wttr.weather[1].to_string())
}

async fn wait_unil_8pm() -> Result<()> {
    let current_time = Local::now();
    debug!("The current time is: {}", current_time);
    #[allow(deprecated)]
    let today_eight_pm = Local::now()
        .date() // Deprecated, but we'll use it since we want to keep the timezone
        .and_hms_opt(20, 0, 0)
        .ok_or_else(|| anyhow::anyhow!("Failed to create today's 8 PM time"))?;

    // We've already passed 8 PM today
    if current_time > today_eight_pm {
        debug!("It's already past 8 PM, waiting until tomorrow");
        tokio::time::sleep(std::time::Duration::from_secs(60 * 60 * 12)).await
    }
    let duration_until_eight_pm = today_eight_pm
        .signed_duration_since(current_time)
        .to_std()?;
    // Wait until 8 PM
    debug!("Waiting for {} seconds", duration_until_eight_pm.as_secs());
    tokio::time::sleep(duration_until_eight_pm).await;
    Ok(())
}

/// Post a toot
fn toot(masto: &Mastodon, status_msg: String) -> Result<()> {
    debug!("Posting new weather update");
    let status = StatusBuilder::new()
        .status(status_msg)
        .visibility(Visibility::Unlisted)
        .language(Language::Swe)
        .content_type("text/html")
        .build();
    let status = match status {
        Ok(status) => status,
        Err(e) => bail!("Error building status: {}", e),
    };
    match masto.new_status(status) {
        Ok(_) => Ok(()),
        Err(e) => bail!("Error posting status: {}", e),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vader_bot=debug".parse().unwrap()),
        )
        .init();
    info!("Starting bot");
    let config: Config = config::Config::new()?;
    let masto = Mastodon::from(config.mastodon.clone());
    match masto.verify_credentials() {
        Ok(_) => (),
        Err(e) => bail!("Error verifying credentials: {}", e),
    };

    loop {
        wait_unil_8pm().await?;
        let weather = fetch_tomorrows_weather(&config.location).await?;
        toot(&masto, weather)?;
    }
}
