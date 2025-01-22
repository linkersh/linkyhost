use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

use super::error::ApiError;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/create", post(create))
        .route("/login", post(login))
}

#[derive(Deserialize, Debug)]
pub struct CreateUserBody {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Debug)]
pub struct CreateUserResp {
    pub token: String,
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateUserBody>,
) -> Result<impl IntoResponse, ApiError> {
    if body.username.len() < 2 || body.password.len() < 2 {
        return Err(ApiError::Validation(
            "username and password must be at least 2 characters long".to_string(),
        ));
    }

    if !state.config.auth.allow_signup {
        return Err(ApiError::Unauthorized("signups are disabled".to_string()));
    }

    let pwd_hash = state.auther.hash_password(body.password).await?;
    let user = state.database.create_user(body.username, pwd_hash).await?;
    let token = state.auther.sign_token(user.id)?;
    Ok(Json(CreateUserResp { token }))
}

pub type LoginUserBody = CreateUserBody;
pub type LoginUserResp = CreateUserResp;

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginUserBody>,
) -> Result<impl IntoResponse, ApiError> {
    if body.username.len() < 2 || body.password.len() < 2 {
        return Err(ApiError::Validation(
            "username and password must be at least 2 characters long".to_string(),
        ));
    }

    let user = state.database.get_user_by_name(body.username).await?;
    let user = match user {
        Some(u) => u,
        None => {
            return Err(ApiError::Unauthorized(
                "invalid username or password".to_owned(),
            ))
        }
    };

    let is_valid = state
        .auther
        .verify_password(body.password, user.password)
        .await?;

    if !is_valid {
        return Err(ApiError::Unauthorized(
            "invalid username or password".to_owned(),
        ));
    }

    let token = state.auther.sign_token(user.id)?;
    Ok(Json(LoginUserResp { token }))
}
