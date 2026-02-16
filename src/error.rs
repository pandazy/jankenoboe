use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use jankensqlhub::{get_error_data, get_error_info, JankenError};
use serde_json::{json, Value};

#[derive(Debug)]
pub enum AppError {
    Database(String),
    InvalidParameter(String),
    NotFound,
    Internal(String),
    Janken(JankenErrorResponse),
}

#[derive(Debug)]
pub struct JankenErrorResponse {
    pub code: u16,
    pub name: String,
    pub description: String,
    pub metadata: Option<Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Janken(err) => {
                let body = Json(json!({
                    "error": {
                        "code": err.code,
                        "name": err.name,
                        "description": err.description,
                        "metadata": err.metadata,
                    }
                }));
                (StatusCode::BAD_REQUEST, body).into_response()
            }
            _ => {
                let (status, message) = match self {
                    AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::InvalidParameter(msg) => (StatusCode::BAD_REQUEST, msg),
                    AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
                    AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::Janken(_) => unreachable!(),
                };

                let body = Json(json!({
                    "error": message,
                }));

                (status, body).into_response()
            }
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<JankenError> for AppError {
    fn from(err: JankenError) -> Self {
        let data = get_error_data(&err);
        let info = get_error_info(data.code);
        let metadata: Option<Value> = data
            .metadata
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        AppError::Janken(JankenErrorResponse {
            code: data.code,
            name: info.map(|i| i.name.to_string()).unwrap_or_default(),
            description: info.map(|i| i.description.to_string()).unwrap_or_default(),
            metadata,
        })
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to JankenError
        if let Some(janken_err) = err.downcast_ref::<JankenError>() {
            let data = get_error_data(janken_err);
            let info = get_error_info(data.code);
            let metadata: Option<Value> = data
                .metadata
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok());

            AppError::Janken(JankenErrorResponse {
                code: data.code,
                name: info.map(|i| i.name.to_string()).unwrap_or_default(),
                description: info.map(|i| i.description.to_string()).unwrap_or_default(),
                metadata,
            })
        } else {
            AppError::Internal(err.to_string())
        }
    }
}
