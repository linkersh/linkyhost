use anyhow::Result;
use chrono::NaiveDateTime;
use std::env;
use uuid::Uuid;

use sqlx::{prelude::FromRow, PgPool, Pool, Postgres};

#[derive(FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct Session {
    pub user_id: Uuid,
    pub session_id: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
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

    pub async fn insert_session(&self, user_id: Uuid, session_id: String) -> Result<Session> {
        let created_session = sqlx::query_as(
            "INSERT INTO sessions (user_id, session_id) VALUES ($1, $2) RETURNING *",
        )
        .bind(user_id)
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(created_session)
    }

    pub async fn fetch_user_by_username(&self, username: &str) -> Result<User> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn fetch_session_id(&self, session_id: &str) -> Result<Session> {
        let session: Session = sqlx::query_as("select * from sessions where session_id = $1")
            .bind(session_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(session)
    }
}
