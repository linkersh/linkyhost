use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::extract::CookieJar;
use ring::rand::{SecureRandom, SystemRandom};
use sqlx::prelude::FromRow;
use std::{env, sync::Arc};
use uuid::Uuid;

use crate::{db::Session, server::ApiState, state::AppState};

#[derive(FromRow)]
struct UserCount {
    count: i64,
}

pub async fn create_default_user(state: &Arc<AppState>) -> Result<()> {
    let username = env::var("DEFAULT_USERNAME")?;
    let password = env::var("DEFAULT_PASSWORD")?;

    if username.len() < 1 {
        panic!("username length too small");
    }

    if password.len() < 8 {
        panic!("password length needs to be >8");
    }

    let row: UserCount = sqlx::query_as("select count(1) as count from users")
        .fetch_one(&state.db.pool)
        .await?;

    if row.count > 0 {
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    tracing::info!("creating default user");
    state.db.insert_user(username, password_hash).await?;

    Ok(())
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let hash = PasswordHash::new(hash).unwrap();

    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok()
}

pub async fn create_session(state: &Arc<AppState>, user_id: Uuid) -> Result<Session> {
    let session_id = create_session_id();
    Ok(state.db.insert_session(user_id, session_id).await?)
}

pub fn create_session_id() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 128];
    rng.fill(&mut bytes).unwrap();

    hex::encode(&bytes)
}

pub struct UserSession(pub Session);

#[async_trait]
impl FromRequestParts<ApiState> for UserSession {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ApiState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar.get("session_id");

        let Some(cookie) = cookie else {
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized"));
        };

        let session = match state.db.fetch_session_id(cookie.value()).await {
            Ok(v) => v,
            Err(error) => {
                tracing::error!(error = ?error, "failed to verify session_id");
                return Err((StatusCode::UNAUTHORIZED, "Unauthorized"));
            }
        };

        Ok(UserSession(session))
    }
}

#[cfg(test)]
mod tests {
    use super::create_session_id;

    #[test]
    fn test_session_id() {
        let session_id = create_session_id();
        assert_eq!(session_id.len(), 256);
    }
}
