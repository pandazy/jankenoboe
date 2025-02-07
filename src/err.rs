// Based on
// https://github.com/tokio-rs/axum/blob/main/examples/anyhow-error-response/src/main.rs
use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde_json::json;

pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let anyhow_message = self.0.to_string();
        let parsed_status_code = parse_http_err_code(&anyhow_message);
        let status = if let Some(status_code) = parsed_status_code {
            status_code
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (
            status,
            Json(json!({
                "error": self.0.to_string(),
                "has_error": true,
                "backtrace": format!("{:?}", self.0.backtrace()),
            })),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn http_err_tag() -> String {
    "http_error".to_owned()
}

pub fn http_err_msg(msg: &str, status_code: StatusCode) -> String {
    format!("{}:{} -=> {}", http_err_tag(), status_code.as_u16(), msg)
}

pub fn parse_http_err_code(msg: &str) -> Option<StatusCode> {
    if msg.contains(&http_err_tag()) {
        let parts: Vec<&str> = msg.split(" -=> ").collect();
        let error_prefix = parts.first()?.to_owned();
        let status_code_str = error_prefix
            .split(':')
            .collect::<Vec<&str>>()
            .get(1)?
            .to_owned();
        return Some(
            status_code_str
                .parse::<StatusCode>()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        );
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_http_err_code() {
        let msg = "http_error:400 -=> test";
        let expected_status_code = StatusCode::BAD_REQUEST;
        assert_eq!(
            parse_http_err_code(msg).unwrap(),
            expected_status_code,
            "status code parsed correctly"
        );
    }

    #[test]
    fn test_http_err_msg() {
        let msg = "test";
        let expected_status_code = StatusCode::BAD_REQUEST;
        let expected_msg = "http_error:400 -=> test";
        let actual_msg = http_err_msg(msg, expected_status_code);
        assert_eq!(actual_msg, expected_msg, "message encoded correctly");
    }
}
