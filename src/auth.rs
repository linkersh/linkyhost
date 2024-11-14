use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use sqlx::prelude::FromRow;
use std::{env, sync::Arc};

use crate::state::AppState;

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
