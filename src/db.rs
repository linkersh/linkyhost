use anyhow::Result;
use chrono::NaiveDateTime;
use uuid::Uuid;
use std::env;

use sqlx::{prelude::FromRow, PgPool, Pool, Postgres};

#[derive(FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

pub struct HostDB {
    pub pool: Pool<Postgres>,
}

impl HostDB {
    pub async fn new() -> Result<HostDB> {
        let database_uri = env::var("POSTGRES_URI")?;
        let pool = PgPool::connect(&database_uri).await?;

        tracing::info!("connection to postgres established");

        Ok(HostDB { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        tracing::info!("running db migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_user(&self, username: String, password: String) -> Result<()> {
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
            .bind(username)
            .bind(password)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn fetch_user_by_username(&self, username: &str) -> Result<User> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }
}
