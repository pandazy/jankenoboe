use serde_json::json;
use std::fmt;

/// Application error type for CLI operations.
/// All errors output JSON to stderr and exit with code 1.
#[derive(Debug)]
pub enum AppError {
    /// Invalid CLI parameter or input
    InvalidParameter(String),
    /// Record not found
    NotFound(String),
    /// Database error
    Database(String),
    /// Internal/unexpected error
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidParameter(msg) => write!(f, "{msg}"),
            AppError::NotFound(msg) => write!(f, "{msg}"),
            AppError::Database(msg) => write!(f, "{msg}"),
            AppError::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InvalidParameter(format!("Invalid JSON: {err}"))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // Try to extract JankenSQLHub error metadata for detailed messages
        if let Some(janken_err) = err.downcast_ref::<jankensqlhub::JankenError>() {
            let data = jankensqlhub::get_error_data(janken_err);
            let param =
                jankensqlhub::error_meta(data, jankensqlhub::M_PARAM_NAME).unwrap_or_default();
            let got = jankensqlhub::error_meta(data, jankensqlhub::M_GOT).unwrap_or_default();
            let info_name = jankensqlhub::get_error_info(data.code)
                .map(|i| i.name.to_string())
                .unwrap_or_else(|| err.to_string());
            return AppError::Internal(format!("{info_name}: param={param}, got={got}"));
        }
        AppError::Internal(err.to_string())
    }
}

/// Print error JSON to stderr and exit with code 1.
pub fn exit_with_error(err: &AppError) -> ! {
    let msg = json!({"error": err.to_string()});
    eprintln!("{msg}");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_invalid_parameter() {
        let err = AppError::InvalidParameter("bad input".to_string());
        assert_eq!(err.to_string(), "bad input");
    }

    #[test]
    fn test_display_not_found() {
        let err = AppError::NotFound("missing".to_string());
        assert_eq!(err.to_string(), "missing");
    }

    #[test]
    fn test_display_database() {
        let err = AppError::Database("db error".to_string());
        assert_eq!(err.to_string(), "db error");
    }

    #[test]
    fn test_display_internal() {
        let err = AppError::Internal("internal error".to_string());
        assert_eq!(err.to_string(), "internal error");
    }

    #[test]
    fn test_from_rusqlite_error() {
        let rusqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let app_err: AppError = rusqlite_err.into();
        assert_eq!(app_err.to_string(), "Query returned no rows");
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let app_err: AppError = json_err.into();
        assert!(app_err.to_string().starts_with("Invalid JSON:"));
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("something broke");
        let app_err: AppError = anyhow_err.into();
        assert_eq!(app_err.to_string(), "something broke");
    }
}
