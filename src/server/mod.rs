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
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::state::AppState;

mod error;
mod images;
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
    let api_router = Router::new()
        .nest("/user", user::router())
        .nest("/images", images::router());

    let service = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_headers([CONTENT_TYPE, CONTENT_LENGTH])
                .allow_credentials(true),
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
