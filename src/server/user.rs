use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;

use crate::{auth, state::AppState};

use super::ApiState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/signin", post(signin))
}

#[derive(Deserialize, Debug)]
pub struct SignInBody {
    username: String,
    password: String,
}

async fn signin(State(state): State<ApiState>, Json(body): Json<SignInBody>) -> String {
    let user = state
        .db
        .fetch_user_by_username(&body.username)
        .await
        .unwrap(); // todo: fix
    let is_valid = auth::verify_password(&user.password, &body.password);

    tracing::info!("password valid: {is_valid}");

    "".to_owned()
}
