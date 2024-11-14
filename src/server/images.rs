use std::sync::Arc;

use axum::Router;
use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
use tempfile::NamedTempFile;

use super::{error::ApiError, ApiState};

pub fn router() -> Router<ApiState> {
    Router::new()
}

#[derive(TryFromMultipart)]
pub struct UploadFilesBody {
    files: Vec<FieldData<NamedTempFile>>,
}

pub async fn upload_files(
    TypedMultipart(multi): TypedMultipart<UploadFilesBody>,
) -> Result<(), ApiError> {
    Ok(())
}
