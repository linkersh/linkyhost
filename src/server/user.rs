use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{
    auth::{self, UserSession},
    state::AppState,
};

use super::{error::ApiError, ApiState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/signin", post(signin))
        .route("/verify", get(verify))
}

#[derive(Deserialize, Debug)]
pub struct SignInBody {
    username: String,
    password: String,
}

async fn signin(
    State(state): State<ApiState>,
    jar: CookieJar,
    Json(body): Json<SignInBody>,
) -> Result<CookieJar, ApiError> {
    let user = state.db.fetch_user_by_username(&body.username).await?;

    let is_valid = auth::verify_password(&user.password, &body.password);
    if !is_valid {
        return Err(ApiError::Unauthorized);
    }

    let session = auth::create_session(&state, user.id).await?;
    let mut cookie = Cookie::new("session_id", session.session_id);

    let mut expires_at = OffsetDateTime::now_utc();
    expires_at += time::Duration::days(30);

    cookie.set_expires(expires_at);
    cookie.set_secure(true);
    cookie.set_http_only(true);
    cookie.set_same_site(SameSite::None);
    cookie.set_partitioned(true);

    Ok(jar.add(cookie))
}

async fn verify(UserSession(session): UserSession) {}
