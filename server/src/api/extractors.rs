use axum::{
    extract::{FromRef, FromRequestParts},
    response::{IntoResponse, Response},
};

use crate::{auth::UserClaims, state::AppState};

use super::error::ApiError;

pub struct ExtractClaims(pub UserClaims);

impl<S> FromRequestParts<S> for ExtractClaims
where
    AppState: FromRef<S>,
    S: Send + Sync + 'static,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let token = parts.headers.get("authorization");
        let token = match token.map(|x| x.to_str()) {
            Some(Ok(a)) => a.to_owned(),
            _ => return Err(ApiError::Unauthorized("Unauthorized".to_owned()).into_response()),
        };

        let claims = match state.auther.verify_token(&token) {
            Ok(c) => c,
            Err(_) => return Err(ApiError::Unauthorized("Unauthorized".to_owned()).into_response()),
        };
        Ok(ExtractClaims(claims))
    }
}
