use crate::error::AppError;

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
    match table {
        "artist" => Ok(&[
            "id",
            "name",
            "name_context",
            "created_at",
            "updated_at",
            "status",
        ]),
        "show" => Ok(&[
            "id",
            "name",
            "name_romaji",
            "vintage",
            "s_type",
            "created_at",
            "updated_at",
            "status",
        ]),
        "song" => Ok(&[
            "id",
            "name",
            "name_context",
            "artist_id",
            "created_at",
            "updated_at",
            "status",
        ]),
        "play_history" => Ok(&[
            "id",
            "show_id",
            "song_id",
            "created_at",
            "media_url",
            "status",
        ]),
        "learning" => Ok(&[
            "id",
            "song_id",
            "level",
            "created_at",
            "updated_at",
            "last_level_up_at",
            "level_up_path",
            "graduated",
        ]),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid table for get: {table}"
        ))),
    }
}

/// Allowed searchable columns per table for the `search` command (--term keys).
pub fn search_columns(table: &str) -> Result<&'static [&'static str], AppError> {
    match table {
        "artist" => Ok(&["name", "name_context"]),
        "show" => Ok(&["name", "vintage"]),
        "song" => Ok(&["name", "name_context", "artist_id"]),
        "play_history" => Ok(&["show_id", "song_id"]),
        "rel_show_song" => Ok(&["show_id", "song_id"]),
        "learning" => Ok(&[
            "song_id",
            "level",
            "graduated",
            "last_level_up_at",
            "level_up_path",
        ]),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid table for search with columns: {table}"
        ))),
    }
}

/// Allowed fields per table for the `search` command (--fields).
pub fn search_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    match table {
        "artist" => Ok(&[
            "id",
            "name",
            "name_context",
            "created_at",
            "updated_at",
            "status",
        ]),
        "show" => Ok(&[
            "id",
            "name",
            "name_romaji",
            "vintage",
            "s_type",
            "created_at",
            "updated_at",
            "status",
        ]),
        "song" => Ok(&[
            "id",
            "name",
            "name_context",
            "artist_id",
            "created_at",
            "updated_at",
            "status",
        ]),
        "play_history" => Ok(&[
            "id",
            "show_id",
            "song_id",
            "created_at",
            "media_url",
            "status",
        ]),
        "rel_show_song" => Ok(&["show_id", "song_id", "media_url", "created_at"]),
        "learning" => Ok(&[
            "id",
            "song_id",
            "level",
            "created_at",
            "updated_at",
            "last_level_up_at",
            "level_up_path",
            "graduated",
        ]),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid table for search with fields: {table}"
        ))),
    }
}

/// Allowed data fields per table for the `create` command (--data keys).
pub fn create_data_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    match table {
        "artist" => Ok(&["name", "name_context"]),
        "show" => Ok(&["name", "name_romaji", "vintage", "s_type"]),
        "song" => Ok(&["name", "name_context", "artist_id"]),
        "play_history" => Ok(&["show_id", "song_id", "media_url"]),
        "learning" => Ok(&["song_id", "level_up_path"]),
        "rel_show_song" => Ok(&["show_id", "song_id", "media_url"]),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid table for create: {table}"
        ))),
    }
}

/// Allowed data fields per table for the `update` command (--data keys).
pub fn update_data_fields(table: &str) -> Result<&'static [&'static str], AppError> {
    match table {
        "artist" => Ok(&["name", "name_context", "status"]),
        "show" => Ok(&["name", "name_romaji", "vintage", "s_type", "status"]),
        "song" => Ok(&["name", "name_context", "artist_id", "status"]),
        "play_history" => Ok(&["show_id", "song_id", "media_url", "status"]),
        "learning" => Ok(&["level", "graduated"]),
        _ => Err(AppError::InvalidParameter(format!(
            "Invalid table for update: {table}"
        ))),
    }
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
    fn test_search_columns_invalid_table() {
        assert_eq!(
            search_columns("bad").unwrap_err().to_string(),
            "Invalid table for search with columns: bad"
        );
    }

    #[test]
    fn test_search_fields_invalid_table() {
        assert_eq!(
            search_fields("bad").unwrap_err().to_string(),
            "Invalid table for search with fields: bad"
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
    fn test_search_columns_all_tables() {
        assert!(search_columns("artist").unwrap().contains(&"name"));
        assert!(search_columns("show").unwrap().contains(&"vintage"));
        assert!(search_columns("song").unwrap().contains(&"artist_id"));
        assert!(search_columns("play_history").unwrap().contains(&"song_id"));
        assert!(
            search_columns("rel_show_song")
                .unwrap()
                .contains(&"show_id")
        );
    }

    #[test]
    fn test_search_fields_all_tables() {
        assert!(search_fields("artist").unwrap().contains(&"name"));
        assert!(search_fields("show").unwrap().contains(&"vintage"));
        assert!(search_fields("song").unwrap().contains(&"artist_id"));
        assert!(
            search_fields("play_history")
                .unwrap()
                .contains(&"media_url")
        );
        assert!(
            search_fields("rel_show_song")
                .unwrap()
                .contains(&"media_url")
        );
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
