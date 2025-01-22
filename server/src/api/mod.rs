use std::time::Duration;

use axum::{
    http::header::{AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, ORIGIN},
    Router,
};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::state::AppState;

pub mod error;
pub mod extractors;
pub mod users;
pub mod vault;

pub struct ApiServer {
    state: AppState,
    cancel: CancellationToken,
}

impl ApiServer {
    pub fn new(state: AppState, cancel: CancellationToken) -> Self {
        ApiServer { state, cancel }
    }

    pub async fn listen(self) -> anyhow::Result<()> {
        let svc_builder = ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(
                CorsLayer::new()
                    .allow_origin(AllowOrigin::list([
                        "http://localhost:5173".parse()?,
                        "http://127.0.0.1:5173".parse()?,
                    ]))
                    .allow_headers([ORIGIN, CONTENT_LENGTH, CONTENT_TYPE, AUTHORIZATION])
                    .expose_headers([CONTENT_LENGTH])
                    .allow_credentials(true)
                    .max_age(Duration::from_secs(60) * 60 * 12),
            );
            // .layer(RequestBodyLimitLayer::new(1024 * 1024 * 1024 * 1024));

        let addy = "127.0.0.1:8080";
        let listener = TcpListener::bind(addy).await?;
        tracing::info!("bound to http://{}", addy);

        let api_router = Router::new()
            .nest("/users", users::router())
            .nest("/vaults", vault::router());

        let router = Router::new()
            .nest("/api", api_router)
            .layer(svc_builder)
            .with_state(self.state);

        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                self.cancel.cancelled().await;
            })
            .await?;

        Ok(())
    }
}
