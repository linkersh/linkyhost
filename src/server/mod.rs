use std::{env, future::Future, sync::Arc};

use axum::{
    http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE},
        HeaderValue,
    },
    Router,
};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::state::AppState;

mod user;

pub type ApiState = Arc<AppState>;

pub async fn create_server(state: &Arc<AppState>) -> anyhow::Result<()> {
    let bind_addr = env::var("BIND_ADDRESS")?;
    let listener = TcpListener::bind(&bind_addr).await?;

    tracing::info!("listening on {bind_addr}");
    axum::serve(listener, make_router(&state))
        .with_graceful_shutdown(make_shutdown(&state))
        .await?;

    Ok(())
}

fn make_router(state: &Arc<AppState>) -> Router {
    let state = Arc::clone(&state);
    let api_router = Router::new().nest("/user", user::router());

    let service = ServiceBuilder::new().layer(
        CorsLayer::new()
            .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
            .allow_headers([CONTENT_TYPE, CONTENT_LENGTH]),
    );

    Router::new()
        .nest("/api", api_router)
        .layer(service)
        .with_state(state)
}

fn make_shutdown(state: &Arc<AppState>) -> impl Future<Output = ()> {
    let state = Arc::clone(&state);
    async move { state.cancel.cancelled().await }
}
