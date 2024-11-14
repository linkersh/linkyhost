use std::sync::Arc;

use db::HostDB;
use state::AppState;
use tokio_util::sync::CancellationToken;

mod auth;
mod db;
mod server;
mod state;

async fn signal(token: CancellationToken) -> anyhow::Result<()> {
    tokio::signal::ctrl_c().await?;
    token.cancel();

    Ok(())
}

fn start_sigint_listener(token: &CancellationToken) {
    let cloned = token.clone();
    tokio::spawn(async move {
        tracing::debug!("SIGINT background listener started");
        signal(cloned).await
    });
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv()?;

    let cancel = CancellationToken::new();
    start_sigint_listener(&cancel);

    let db = HostDB::new().await?;
    db.migrate().await?;

    let state = Arc::new(AppState { db, cancel });
    auth::create_default_user(&state).await?;
    server::create_server(&state).await?;

    Ok(())
}
