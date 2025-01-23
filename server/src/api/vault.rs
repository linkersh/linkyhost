use std::time::Instant;

use axum::{
    body::Body,
    extract::{DefaultBodyLimit, Path, Query, State},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, BufReader},
};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{db::VaultFile, state::AppState, store::{ThumbOptions, Thumbnail}, uploader::CreateUploadInfo};

use super::{error::ApiError, extractors::ExtractClaims};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_vaults))
        .route("/create", post(create_vault))
        .route("/{id}/beginUpload", post(start_upload))
        .route("/uploads/{id}/chunk", post(upload_chunk))
        .layer(DefaultBodyLimit::disable())
        .route("/{id}/delete", post(delete_vault))
        .route("/{vaultId}/files", get(list_files))
        .route("/{vaultId}/files/{fileId}/thumbnail", get(file_thumb))
}

#[derive(Deserialize, Debug)]
pub struct CreateVaultInfo {
    name: String,
    flags: i32,
}

pub async fn create_vault(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Json(body): Json<CreateVaultInfo>,
) -> Result<impl IntoResponse, ApiError> {
    if body.name.len() < 2 {
        return Err(ApiError::Validation("Vault name is too short".to_string()));
    }
    let vault = state
        .database
        .create_vault(claims.sub, body.name, body.flags)
        .await?;
    Ok(Json(vault))
}

pub async fn list_vaults(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let vaults = state.database.list_vaults(claims.sub).await?;
    Ok(Json(vaults))
}

pub async fn delete_vault(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    state.database.delete_vault(claims.sub, id).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct StartUploadBody {
    file_name: String,
    file_size: u64,
    content_type: String,
}

#[derive(Serialize)]
pub struct StartUploadResp {
    id: Uuid,
}

pub async fn start_upload(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Path(vault_id): Path<i64>,
    Json(body): Json<StartUploadBody>,
) -> Result<impl IntoResponse, ApiError> {
    let vault = state.database.get_vault_by_id(claims.sub, vault_id).await?;
    if vault.is_none() {
        return Err(ApiError::NotFound(
            "Vault not found or you dont own it".to_string(),
        ));
    }

    let id = Uuid::new_v4();
    let temp_file = state.store.create_temp_file(id).await?;
    state
        .uploader
        .create_upload(CreateUploadInfo {
            file_name: body.file_name,
            file_size: body.file_size as usize,
            file_type: body.content_type,
            tmp_file: temp_file,
            upload_id: id,
            user_id: claims.sub,
            vault_id,
        })
        .await?;
    Ok(Json(StartUploadResp { id }))
}

#[derive(TryFromMultipart)]
pub struct UploadChunkBody {
    #[form_data(limit = "150MB")]
    data: FieldData<NamedTempFile>,
}

pub async fn upload_chunk(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Path(upload_id): Path<Uuid>,
    TypedMultipart(UploadChunkBody { data }): TypedMultipart<UploadChunkBody>,
) -> Result<impl IntoResponse, ApiError> {
    let upload = state.uploader.get_upload(upload_id).await;
    let upload = match upload {
        Some(up) => up,
        None => {
            return Err(ApiError::NotFound("Upload not found".to_string()));
        }
    };

    let mut upload = upload.lock().await;

    if upload.user_id != claims.sub {
        return Err(ApiError::NotFound("Upload not found".to_string()));
    }

    let async_handle = OpenOptions::new()
        .read(true)
        .open(data.contents.path())
        .await?;

    let mut reader = BufReader::new(async_handle);
    let mut buf = Vec::with_capacity(8192);

    loop {
        let n = reader.read_buf(&mut buf).await?;
        if n == 0 {
            break;
        }

        upload.received_size += n;
        upload.last_received = Utc::now();
        if upload.received_size > upload.expected_size {
            return Err(ApiError::Validation("too many bytes sent".to_owned()));
        }

        upload.tmp_file.write_all(&buf).await?;
        buf.clear();
    }

    println!("{} cursor at", upload.tmp_file.stream_position().await?);
    upload.tmp_file.flush().await?;
    if upload.received_size == upload.expected_size {
        println!("received all bytes: {}", upload.expected_size);

        let file = state
            .database
            .create_file(
                &upload.file_name,
                &upload.content_type,
                upload.received_size as i64,
                upload.vault_id,
                upload.user_id,
            )
            .await?;

        let upload_start = Instant::now();

        state
            .store
            .upload_file(upload.vault_id, file.id as i64, upload.id)
            .await?;

        println!("took: {:.2?} to upload", upload_start.elapsed());
    }

    drop(upload);
    drop(reader);

    data.contents.close()?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ListFilesQuery {
    limit: u64,
    skip: u64,
}

pub async fn list_files(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Path(vault_id): Path<i64>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<VaultFile>>, ApiError> {
    let vault = state.database.get_vault_by_id(claims.sub, vault_id).await?;
    if vault.is_none() {
        return Err(ApiError::NotFound(
            "Vault not found or you dont own it".to_string(),
        ));
    }

    let files = state
        .database
        .list_files(claims.sub, vault_id, query.limit as i64, query.skip as i64)
        .await?;
    Ok(Json(files))
}

pub async fn file_thumb(
    ExtractClaims(claims): ExtractClaims,
    State(state): State<AppState>,
    Path((vault_id, file_id)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, ApiError> {
    let vault = state.database.get_vault_by_id(claims.sub, vault_id).await?;
    if vault.is_none() {
        return Err(ApiError::NotFound(
            "Vault not found or you dont own it".to_string(),
        ));
    }

    const MAX_FILE_SIZE: i64 = 1024 * 1024 * 50;
    let file = state.database.get_file_by_id(vault_id, file_id).await?;
    if !file.content_type.starts_with("image/") {
        return Err(ApiError::UnsupportedFileType);
    }

    // we avoid generating thumbnails for files >50MB
    if file.size > MAX_FILE_SIZE {
        return Err(ApiError::UnsupportedFileType);
    }

    let options = ThumbOptions {
        width: 256,
        height: 256,
        vault_id,
        file_id,
    };

    let thumbnail = Thumbnail::new(options, state.clone());
    let thumb_path = thumbnail.process().await?;
    let thumb_file = File::open(thumb_path).await?;
    let body = Body::from_stream(ReaderStream::new(thumb_file));
    let res = Response::builder()
        .header("Content-Type", file.content_type)
        .body(body)?;

    Ok(res)
}
