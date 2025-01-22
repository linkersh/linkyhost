use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found {0}")]
    NotFound(String),
    #[error("Internal Server Error")]
    Internal(#[from] anyhow::Error),
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        Self::Internal(value.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::Validation(m) => (StatusCode::BAD_REQUEST, m.to_string()),
            Self::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.to_string()),
            Self::NotFound(m) => (StatusCode::NOT_FOUND, m.to_string()),
            Self::Internal(error) => {
                tracing::error!(error = ?error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error".to_string(),
                )
            }
        };

        let body = Json(serde_json::json!({ "message": message }));
        (status, body).into_response()
    }
}
