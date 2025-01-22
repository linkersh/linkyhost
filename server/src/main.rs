use std::path::Path;

use anyhow::Result;
use api::ApiServer;
use config::AppConfig;
use tokio_util::sync::CancellationToken;

mod api;
mod auth;
mod config;
mod db;
mod state;
mod uploader;
mod store;

async fn shutdown_signal() -> anyhow::Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cancel = CancellationToken::new();
    let config = AppConfig::load_from_file(Path::new("config.toml"))?;
    let state = state::create_state(config).await?;

    let server = ApiServer::new(state, cancel.clone());
    let listen_task = server.listen();
    let shutdown_task = shutdown_signal();

    tokio::select! {
        _ = listen_task => {},
        _ = shutdown_task => {
            cancel.cancel();
            tracing::info!("shutdown signal received");
        }
    }

    Ok(())
}
