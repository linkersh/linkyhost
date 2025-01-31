use std::path::{Path, PathBuf};

use anyhow::Result;
use api::ApiServer;
use chrono::Utc;
use config::AppConfig;
use storage::chunk::{Chunk, ChunkInfo};
use tokio_util::sync::CancellationToken;

mod api;
mod auth;
mod config;
mod db;
mod state;
mod storage;
mod store;
mod uploader;

async fn shutdown_signal() -> anyhow::Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // tracing_subscriber::fmt::init();

    // let cancel = CancellationToken::new();
    // let config = AppConfig::load_from_file(Path::new("config.toml"))?;
    // let state = state::create_state(config, cancel.clone()).await?;

    // let server = ApiServer::new(state, cancel.clone());
    // let listen_task = server.listen();
    // let shutdown_task = shutdown_signal();

    // tokio::select! {
    //     res = listen_task => {
    //         if let Err(error) = res {
    //             tracing::error!(error = ?error, "failed to listen");
    //         }
    //     },
    //     _ = shutdown_task => {
    //         cancel.cancel();
    //         tracing::info!("shutdown signal received");
    //     }
    // }

    // let chunk = Chunk::write_new(
    //     PathBuf::from("./chunk.bin"),
    //     ChunkInfo {
    //         created_at: Utc::now(),
    //         id: 1,
    //     },
    // )
    // .await?;

    let chunk = Chunk::read_from("./chunk.bin").await?;
    dbg!(chunk);

    Ok(())
}
