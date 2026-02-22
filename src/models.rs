use crate::error::AppError;
use crate::table_config;

/// Valid table names for the `get` command.
pub const GET_TABLES: &[&str] = &["artist", "show", "song", "play_history", "learning"];

/// Valid table names for the `search` command.
pub const SEARCH_TABLES: &[&str] = &[
    "artist",
    "show",
    "song",
    "play_history",
    "rel_show_song",
    "learning",
];

/// Valid table names for the `duplicates` command.
pub const DUPLICATES_TABLES: &[&str] = &["artist", "show", "song"];

/// Valid table names for the `create` command.
pub const CREATE_TABLES: &[&str] = &[
    "artist",
    "show",
    "song",
    "play_history",
    "learning",
    "rel_show_song",
];

/// Valid table names for the `update` command.
pub const UPDATE_TABLES: &[&str] = &["artist", "show", "song", "play_history", "learning"];

/// Valid table names for the `delete` command.
pub const DELETE_TABLES: &[&str] = &["artist", "song"];

/// Allowed fields per table for the `get` command (--fields).
pub fn get_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    table_config::get(table)
        .map(|c| c.selectable)
        .ok_or_else(|| AppError::InvalidParameter(format!("Invalid table for get: {table}")))
}

/// Allowed data fields per table for the `create` command (--data keys).
pub fn create_data_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    table_config::get(table)
        .map(|c| c.creatable)
        .ok_or_else(|| AppError::InvalidParameter(format!("Invalid table for create: {table}")))
}

/// Allowed data fields per table for the `update` command (--data keys).
pub fn update_data_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    table_config::get(table)
        .map(|c| c.updatable)
        .ok_or_else(|| AppError::InvalidParameter(format!("Invalid table for update: {table}")))
}

/// Valid match modes for search term conditions.
pub const MATCH_MODES: &[&str] = &["exact", "exact-i", "starts-with", "ends-with", "contains"];

/// Validate a table name against an allowed list.
pub fn validate_table(table: &str, allowed: &[&str]) -> Result<(), AppError> {
    if allowed.contains(&table) {
        Ok(())
    } else {
        Err(AppError::InvalidParameter(format!(
            "Invalid table: {table}. Allowed: {}",
            allowed.join(", ")
        )))
    }
}

/// Validate that all requested fields are allowed for a given table.
pub fn validate_fields(fields: &[String], allowed: &[&str]) -> Result<(), AppError> {
    for f in fields {
        if !allowed.contains(&f.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "Invalid field: {f}. Allowed: {}",
                allowed.join(", ")
            )));
        }
    }
    Ok(())
}

/// Parse a comma-separated fields string into a vector.
pub fn parse_fields(fields_str: &str) -> Vec<String> {
    fields_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Get the current Unix timestamp in seconds.
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fields_invalid_table() {
        assert_eq!(
            get_fields("bad").unwrap_err().to_string(),
            "Invalid table for get: bad"
        );
    }

    #[test]
    fn test_create_data_fields_invalid_table() {
        assert_eq!(
            create_data_fields("bad").unwrap_err().to_string(),
            "Invalid table for create: bad"
        );
    }

    #[test]
    fn test_update_data_fields_invalid_table() {
        assert_eq!(
            update_data_fields("bad").unwrap_err().to_string(),
            "Invalid table for update: bad"
        );
    }

    #[test]
    fn test_get_fields_all_tables() {
        assert!(get_fields("artist").unwrap().contains(&"name"));
        assert!(get_fields("show").unwrap().contains(&"vintage"));
        assert!(get_fields("song").unwrap().contains(&"artist_id"));
        assert!(get_fields("play_history").unwrap().contains(&"media_url"));
        assert!(get_fields("learning").unwrap().contains(&"level"));
    }

    #[test]
    fn test_create_data_fields_all_tables() {
        assert!(create_data_fields("artist").unwrap().contains(&"name"));
        assert!(create_data_fields("show").unwrap().contains(&"vintage"));
        assert!(create_data_fields("song").unwrap().contains(&"artist_id"));
        assert!(
            create_data_fields("play_history")
                .unwrap()
                .contains(&"media_url")
        );
        assert!(create_data_fields("learning").unwrap().contains(&"song_id"));
        assert!(
            create_data_fields("rel_show_song")
                .unwrap()
                .contains(&"show_id")
        );
    }

    #[test]
    fn test_update_data_fields_all_tables() {
        assert!(update_data_fields("artist").unwrap().contains(&"status"));
        assert!(update_data_fields("show").unwrap().contains(&"status"));
        assert!(update_data_fields("song").unwrap().contains(&"status"));
        assert!(
            update_data_fields("play_history")
                .unwrap()
                .contains(&"status")
        );
        assert!(update_data_fields("learning").unwrap().contains(&"level"));
    }
}
