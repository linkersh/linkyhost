use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::chrono::NaiveDateTime, Executor, Pool, Postgres};

use crate::config::AppConfig;

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

#[derive(FromRow, Serialize)]
pub struct Vault {
    pub id: i64,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub user_id: i32,
    pub is_encrypted: bool,
}

#[derive(FromRow, Serialize, Deserialize)]
pub struct VaultFile {
    pub id: i32,
    pub vault_id: i32,
    pub user_id: i32,
    pub file_name: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
    pub uploaded_at: NaiveDateTime,
    pub content_type: String,
    pub fixed_iv: Vec<u8>,
    pub password_salt: Vec<u8>,
    pub chunk_size: i32,
    pub is_encrypted: bool,
    pub is_hidden: bool,
}

pub struct CreateVaultFileOpts {
    pub file_name: String,
    pub size: i64,
    pub content_type: String,
    pub salt: Vec<u8>,
    pub fixed_iv: Vec<u8>,
    pub is_hidden: bool,
    pub is_encrypted: bool,
    pub chunk_size: i32,
    pub vault_id: i32,
    pub user_id: i32,
}

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(config: &AppConfig) -> Result<Database> {
        let pool = sqlx::postgres::PgPool::connect(&config.db_uri).await?;

        pool.execute("SELECT 1").await?;
        Ok(Database { pool })
    }

    pub async fn create_user(&self, username: String, password: String) -> Result<User> {
        let user: User =
            sqlx::query_as("INSERT INTO users (username, password) VALUES ($1, $2) RETURNING *")
                .bind(username)
                .bind(password)
                .fetch_one(&self.pool)
                .await?;

        Ok(user)
    }

    pub async fn get_file_by_id(&self, vault_id: i64, file_id: i64) -> Result<VaultFile> {
        let file: VaultFile =
            sqlx::query_as("select * from vault_files where vault_id = $1 and id = $2")
                .bind(vault_id)
                .bind(file_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(file)
    }

    pub async fn get_user_by_name(&self, username: String) -> Result<Option<User>> {
        let user: Option<User> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn create_vault(
        &self,
        user_id: i32,
        name: String,
        is_encrypted: bool,
    ) -> Result<Vault> {
        let vault: Vault = sqlx::query_as(
            "INSERT INTO vaults (name, user_id, is_encrypted) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(name)
        .bind(user_id)
        .bind(is_encrypted)
        .fetch_one(&self.pool)
        .await?;

        Ok(vault)
    }

    pub async fn create_file(&self, options: CreateVaultFileOpts) -> Result<VaultFile> {
        let file: VaultFile = sqlx::query_as(
            "INSERT INTO 
                vault_files (
                    file_name, size, vault_id, user_id, 
                    created_at, content_type, password_salt, fixed_iv,
                    is_hidden, is_encrypted, chunk_size
                )
                VALUES ($1, $2, $3, $4, NOW(), $5, $6, $7, $8, $9, $10) 
                RETURNING *
            ",
        )
        .bind(options.file_name)
        .bind(options.size)
        .bind(options.vault_id)
        .bind(options.user_id)
        .bind(options.content_type)
        .bind(options.salt)
        .bind(options.fixed_iv)
        .bind(options.is_hidden)
        .bind(options.is_encrypted)
        .bind(options.chunk_size)
        .fetch_one(&self.pool)
        .await?;

        Ok(file)
    }

    pub async fn list_vaults(&self, user_id: i32) -> Result<Vec<Vault>> {
        let vaults: Vec<Vault> = sqlx::query_as("SELECT * FROM vaults WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(vaults)
    }

    pub async fn list_files(
        &self,
        user_id: i32,
        vault_id: i64,
        limit: i64,
        skip: i64,
    ) -> Result<Vec<VaultFile>> {
        let files: Vec<VaultFile> = sqlx::query_as(
            "SELECT * FROM vault_files WHERE user_id = $1 and vault_id = $2 LIMIT $3 OFFSET $4",
        )
        .bind(user_id)
        .bind(vault_id)
        .bind(limit)
        .bind(skip)
        .fetch_all(&self.pool)
        .await?;

        Ok(files)
    }

    pub async fn delete_vault(&self, user_id: i32, vault_id: i64) -> Result<()> {
        sqlx::query("DELETE FROM vault_files WHERE vault_id = $1")
            .bind(vault_id)
            .execute(&self.pool)
            .await?;

        sqlx::query("DELETE FROM vaults WHERE user_id = $1 AND id = $2")
            .bind(user_id)
            .bind(vault_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_vault_by_id(&self, user_id: i32, vault_id: i64) -> Result<Option<Vault>> {
        let vault: Option<Vault> =
            sqlx::query_as("SELECT * FROM vaults WHERE user_id = $1 AND id = $2")
                .bind(user_id)
                .bind(vault_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(vault)
    }
}
