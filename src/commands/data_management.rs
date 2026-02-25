use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Map, Value, json};

use crate::easing::{MAX_LEVEL, generate_level_up_path_json};
use crate::encoding::url_decode;
use crate::error::AppError;
use crate::models;
use crate::table_config;

// ---------------------------------------------------------------------------
// create <table> --data
// ---------------------------------------------------------------------------

pub fn cmd_create(conn: &mut Connection, table: &str, data_json: &str) -> Result<Value, AppError> {
    models::validate_table(table, models::CREATE_TABLES)?;

    let mut data: Map<String, Value> = serde_json::from_str(data_json)?;
    url_decode_map_values(&mut data)?;
    let allowed = models::create_data_fields(table)?;

    // Validate all keys
    for key in data.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "Invalid field for create {table}: {key}. Allowed: {}",
                allowed.join(", ")
            )));
        }
    }

    let now = models::now_unix();

    // rel_show_song has no id column - use composite key
    if table == "rel_show_song" {
        return create_rel_show_song(conn, &data, now);
    }

    let id = uuid::Uuid::new_v4().to_string();

    // Build columns, placeholders, args, and param values dynamically
    let mut columns: Vec<String> = vec!["\"id\"".to_string()];
    let mut placeholders: Vec<String> = vec!["@p_id".to_string()];
    let mut args = serde_json::Map::new();
    let mut param_values = serde_json::Map::new();

    args.insert("p_id".to_string(), json!({})); // defaults to string
    param_values.insert("p_id".to_string(), json!(&id));

    // Add user-provided fields (skip nulls — let DB defaults apply)
    for key in allowed {
        if let Some(val) = data.get(*key) {
            if val.is_null() {
                continue;
            }
            let param_key = format!("p_{key}");
            columns.push(format!("\"{key}\""));
            placeholders.push(format!("@{param_key}"));
            let (arg_def, param_val) = json_value_to_param(val);
            args.insert(param_key.clone(), arg_def);
            param_values.insert(param_key, param_val);
        }
    }

    // Auto-add created_at
    add_integer_column(
        &mut columns,
        &mut placeholders,
        &mut args,
        &mut param_values,
        "created_at",
        now,
    );

    // Auto-add updated_at for tables that have it
    if matches!(table, "artist" | "show" | "song" | "learning") {
        add_integer_column(
            &mut columns,
            &mut placeholders,
            &mut args,
            &mut param_values,
            "updated_at",
            now,
        );
    }

    // Learning-specific defaults
    if table == "learning" {
        if !data.contains_key("level") {
            add_integer_column(
                &mut columns,
                &mut placeholders,
                &mut args,
                &mut param_values,
                "level",
                0,
            );
        }
        add_integer_column(
            &mut columns,
            &mut placeholders,
            &mut args,
            &mut param_values,
            "last_level_up_at",
            0,
        );
        if !data.contains_key("graduated") {
            add_integer_column(
                &mut columns,
                &mut placeholders,
                &mut args,
                &mut param_values,
                "graduated",
                0,
            );
        }
        if !data.contains_key("level_up_path") {
            let param_key = "p_level_up_path";
            columns.push("\"level_up_path\"".to_string());
            placeholders.push(format!("@{param_key}"));
            args.insert(param_key.to_string(), json!({}));
            param_values.insert(
                param_key.to_string(),
                json!(generate_level_up_path_json(MAX_LEVEL)),
            );
        }
    }

    let cols_sql = columns.join(", ");
    let placeholders_sql = placeholders.join(", ");
    let query_sql = format!("INSERT INTO #[table] ({cols_sql}) VALUES ({placeholders_sql})");

    args.insert(
        "table".to_string(),
        json!({"enum": table_config::build_table_enum(models::CREATE_TABLES)}),
    );
    param_values.insert("table".to_string(), json!(table));

    let query_json = json!({
        "create_record": {
            "query": query_sql,
            "args": args
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    jankensqlhub::query_run_sqlite(conn, &queries, "create_record", &json!(param_values))
        .map_err(AppError::from)?;

    Ok(json!({"id": id}))
}

/// Create a rel_show_song record (no id column, composite key).
fn create_rel_show_song(
    conn: &mut Connection,
    data: &Map<String, Value>,
    now: i64,
) -> Result<Value, AppError> {
    let show_id = data
        .get("show_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidParameter("show_id is required".into()))?;
    let song_id = data
        .get("song_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidParameter("song_id is required".into()))?;
    let media_url = data.get("media_url").and_then(|v| v.as_str()).unwrap_or("");

    let query_json = json!({
        "create_rel": {
            "query": "INSERT INTO rel_show_song (show_id, song_id, media_url, created_at) VALUES (@show_id, @song_id, @media_url, @now)",
            "args": {
                "show_id": {},
                "song_id": {},
                "media_url": {},
                "now": {"type": "integer"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!({
        "show_id": show_id,
        "song_id": song_id,
        "media_url": media_url,
        "now": now
    });

    jankensqlhub::query_run_sqlite(conn, &queries, "create_rel", &params)
        .map_err(AppError::from)?;

    Ok(json!({"id": format!("{show_id}:{song_id}")}))
}

// ---------------------------------------------------------------------------
// update <table> <id> --data
// ---------------------------------------------------------------------------

pub fn cmd_update(
    conn: &mut Connection,
    table: &str,
    id: &str,
    data_json: &str,
) -> Result<Value, AppError> {
    models::validate_table(table, models::UPDATE_TABLES)?;

    let mut data: Map<String, Value> = serde_json::from_str(data_json)?;
    url_decode_map_values(&mut data)?;
    if data.is_empty() {
        return Err(AppError::InvalidParameter("data cannot be empty".into()));
    }
    let allowed = models::update_data_fields(table)?;

    for key in data.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "Invalid field for update {table}: {key}. Allowed: {}",
                allowed.join(", ")
            )));
        }
    }

    let now = models::now_unix();
    let level_changed = table == "learning" && data.contains_key("level");

    // Build SET clauses, args, and param values dynamically
    let mut set_parts: Vec<String> = Vec::new();
    let mut args = serde_json::Map::new();
    let mut param_values = serde_json::Map::new();

    for key in allowed {
        if let Some(val) = data.get(*key) {
            let param_key = format!("p_{key}");
            set_parts.push(format!("\"{key}\"=@{param_key}"));
            let (arg_def, param_val) = json_value_to_param(val);
            args.insert(param_key.clone(), arg_def);
            param_values.insert(param_key, param_val);
        }
    }

    // Auto-update updated_at for tables that have it
    if matches!(table, "artist" | "show" | "song" | "learning") {
        let param_key = "p_updated_at";
        set_parts.push(format!("\"updated_at\"=@{param_key}"));
        args.insert(param_key.to_string(), json!({"type": "integer"}));
        param_values.insert(param_key.to_string(), json!(now));
    }

    // Auto-update last_level_up_at when level changes
    if level_changed {
        let param_key = "p_last_level_up_at";
        set_parts.push(format!("\"last_level_up_at\"=@{param_key}"));
        args.insert(param_key.to_string(), json!({"type": "integer"}));
        param_values.insert(param_key.to_string(), json!(now));
    }

    let set_sql = set_parts.join(", ");

    // Add table and id params
    args.insert(
        "table".to_string(),
        json!({"enum": table_config::build_table_enum(models::UPDATE_TABLES)}),
    );
    args.insert("id".to_string(), json!({}));
    param_values.insert("table".to_string(), json!(table));
    param_values.insert("id".to_string(), json!(id));

    let query_json = json!({
        "check_exists": {
            "query": "SELECT id FROM #[table] WHERE id=@id",
            "returns": ["id"],
            "args": {
                "table": {"enum": table_config::build_table_enum(models::UPDATE_TABLES)},
                "id": {}
            }
        },
        "update_record": {
            "query": format!("UPDATE #[table] SET {set_sql} WHERE id=@id"),
            "args": args
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!(param_values);

    // Check existence first
    let check = jankensqlhub::query_run_sqlite(conn, &queries, "check_exists", &params)
        .map_err(AppError::from)?;

    if check.data.is_empty() {
        return Err(AppError::NotFound(format!(
            "Record not found: {table}/{id}"
        )));
    }

    jankensqlhub::query_run_sqlite(conn, &queries, "update_record", &params)
        .map_err(AppError::from)?;

    Ok(json!({"updated": true}))
}

// ---------------------------------------------------------------------------
// delete <table> <id>
// ---------------------------------------------------------------------------

pub fn cmd_delete(conn: &mut Connection, table: &str, id: &str) -> Result<Value, AppError> {
    models::validate_table(table, models::DELETE_TABLES)?;

    let query_json = json!({
        "check_exists": {
            "query": "SELECT id FROM #[table] WHERE id=@id",
            "returns": ["id"],
            "args": {
                "table": {"enum": table_config::build_table_enum(models::DELETE_TABLES)},
                "id": {}
            }
        },
        "delete_by_id": {
            "query": "DELETE FROM #[table] WHERE id=@id",
            "args": {
                "table": {"enum": table_config::build_table_enum(models::DELETE_TABLES)},
                "id": {}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!({ "table": table, "id": id });

    let check = jankensqlhub::query_run_sqlite(conn, &queries, "check_exists", &params)
        .map_err(AppError::from)?;

    if check.data.is_empty() {
        return Err(AppError::NotFound(format!(
            "Record not found: {table}/{id}"
        )));
    }

    jankensqlhub::query_run_sqlite(conn, &queries, "delete_by_id", &params)
        .map_err(AppError::from)?;

    Ok(json!({"deleted": true}))
}

// ---------------------------------------------------------------------------
// bulk-reassign (by song IDs or by source artist)
// ---------------------------------------------------------------------------

pub fn cmd_bulk_reassign(
    conn: &mut Connection,
    song_ids_str: Option<&str>,
    new_artist_id: Option<&str>,
    from_artist_id: Option<&str>,
    to_artist_id: Option<&str>,
) -> Result<Value, AppError> {
    // Mode 1: --song-ids + --new-artist-id
    // Mode 2: --from-artist-id + --to-artist-id
    match (song_ids_str, new_artist_id, from_artist_id, to_artist_id) {
        (Some(ids_str), Some(new_id), None, None) => {
            let ids: Vec<&str> = ids_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if ids.is_empty() {
                return Err(AppError::InvalidParameter(
                    "song_ids cannot be empty".into(),
                ));
            }

            let query_json = json!({
                "count_songs": {
                    "query": "SELECT COUNT(*) as cnt FROM song WHERE id IN :[song_ids]",
                    "returns": ["cnt"],
                    "args": {
                        "song_ids": {"itemtype": "string"}
                    }
                },
                "reassign_by_ids": {
                    "query": "UPDATE song SET artist_id=@new_artist_id, updated_at=@now WHERE id IN :[song_ids]",
                    "args": {
                        "new_artist_id": {},
                        "now": {"type": "integer"},
                        "song_ids": {"itemtype": "string"}
                    }
                }
            });

            let queries = QueryDefinitions::from_json(query_json)
                .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

            let ids_json: Vec<Value> = ids.iter().map(|s| json!(s)).collect();

            // Count existing songs to report accurate reassigned_count
            let count_params = json!({ "song_ids": ids_json });
            let count_result =
                jankensqlhub::query_run_sqlite(conn, &queries, "count_songs", &count_params)
                    .map_err(AppError::from)?;
            let count = count_result.data[0]["cnt"].as_i64().unwrap_or(0);

            // Execute the reassignment
            let params = json!({
                "new_artist_id": new_id,
                "now": models::now_unix(),
                "song_ids": ids_json
            });
            jankensqlhub::query_run_sqlite(conn, &queries, "reassign_by_ids", &params)
                .map_err(AppError::from)?;

            Ok(json!({"reassigned_count": count}))
        }
        (None, None, Some(from_id), Some(to_id)) => {
            let query_json = json!({
                "count_by_artist": {
                    "query": "SELECT COUNT(*) as cnt FROM song WHERE artist_id=@from_artist_id",
                    "returns": ["cnt"],
                    "args": {
                        "from_artist_id": {}
                    }
                },
                "reassign_by_artist": {
                    "query": "UPDATE song SET artist_id=@to_artist_id, updated_at=@now WHERE artist_id=@from_artist_id",
                    "args": {
                        "to_artist_id": {},
                        "now": {"type": "integer"},
                        "from_artist_id": {}
                    }
                }
            });

            let queries = QueryDefinitions::from_json(query_json)
                .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

            // Count matching songs first
            let count_params = json!({ "from_artist_id": from_id });
            let count_result =
                jankensqlhub::query_run_sqlite(conn, &queries, "count_by_artist", &count_params)
                    .map_err(AppError::from)?;
            let count = count_result.data[0]["cnt"].as_i64().unwrap_or(0);

            // Execute the reassignment
            let params = json!({
                "to_artist_id": to_id,
                "now": models::now_unix(),
                "from_artist_id": from_id
            });
            jankensqlhub::query_run_sqlite(conn, &queries, "reassign_by_artist", &params)
                .map_err(AppError::from)?;

            Ok(json!({"reassigned_count": count}))
        }
        _ => Err(AppError::InvalidParameter(
            "bulk-reassign requires either (--song-ids + --new-artist-id) or (--from-artist-id + --to-artist-id)"
                .into(),
        )),
    }
}

// ---------------------------------------------------------------------------
// Helpers (local to data_management)
// ---------------------------------------------------------------------------

/// URL-decode all string values in a JSON object map.
/// Non-string values (numbers, booleans, nulls, arrays, objects) are left unchanged.
fn url_decode_map_values(data: &mut Map<String, Value>) -> Result<(), AppError> {
    for (key, val) in data.iter_mut() {
        if let Value::String(s) = val {
            let decoded = url_decode(s).map_err(|e| {
                AppError::InvalidParameter(format!("URL decoding error for field '{key}': {e}"))
            })?;
            *val = Value::String(decoded);
        }
    }
    Ok(())
}

/// Convert a serde_json::Value to a JankenSQLHub (arg_definition, param_value) pair.
/// - String → default string arg
/// - Number (i64) → integer arg
/// - Number (f64) → float arg
/// - Bool → integer arg (true=1, false=0)
/// - Array/Object → default string arg (serialized to string)
fn json_value_to_param(val: &Value) -> (Value, Value) {
    match val {
        Value::String(_) => (json!({}), val.clone()),
        Value::Number(n) => {
            if n.is_i64() {
                (json!({"type": "integer"}), val.clone())
            } else {
                (json!({"type": "float"}), val.clone())
            }
        }
        Value::Bool(b) => (json!({"type": "integer"}), json!(if *b { 1 } else { 0 })),
        _ => (json!({}), json!(val.to_string())),
    }
}

/// Add an integer column to the dynamic INSERT builder.
fn add_integer_column(
    columns: &mut Vec<String>,
    placeholders: &mut Vec<String>,
    args: &mut serde_json::Map<String, Value>,
    param_values: &mut serde_json::Map<String, Value>,
    col_name: &str,
    value: i64,
) {
    let param_key = format!("p_{col_name}");
    columns.push(format!("\"{col_name}\""));
    placeholders.push(format!("@{param_key}"));
    args.insert(param_key.clone(), json!({"type": "integer"}));
    param_values.insert(param_key, json!(value));
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- json_value_to_param ---

    #[test]
    fn test_json_value_to_param_string() {
        let (arg, val) = json_value_to_param(&json!("hello"));
        assert_eq!(arg, json!({}));
        assert_eq!(val, json!("hello"));
    }

    #[test]
    fn test_json_value_to_param_integer() {
        let (arg, val) = json_value_to_param(&json!(42));
        assert_eq!(arg, json!({"type": "integer"}));
        assert_eq!(val, json!(42));
    }

    #[test]
    fn test_json_value_to_param_float() {
        let (arg, val) = json_value_to_param(&json!(3.14));
        assert_eq!(arg, json!({"type": "float"}));
        assert_eq!(val, json!(3.14));
    }

    #[test]
    fn test_json_value_to_param_bool_true() {
        let (arg, val) = json_value_to_param(&json!(true));
        assert_eq!(arg, json!({"type": "integer"}));
        assert_eq!(val, json!(1));
    }

    #[test]
    fn test_json_value_to_param_bool_false() {
        let (arg, val) = json_value_to_param(&json!(false));
        assert_eq!(arg, json!({"type": "integer"}));
        assert_eq!(val, json!(0));
    }

    #[test]
    fn test_json_value_to_param_array() {
        let arr = json!([1, 2, 3]);
        let (arg, val) = json_value_to_param(&arr);
        assert_eq!(arg, json!({}));
        assert_eq!(val, json!("[1,2,3]"));
    }

    #[test]
    fn test_json_value_to_param_null() {
        let (arg, val) = json_value_to_param(&json!(null));
        assert_eq!(arg, json!({}));
        assert_eq!(val, json!("null"));
    }

    // --- url_decode_map_values ---

    #[test]
    fn test_url_decode_map_values_string_decoded() {
        let mut map = serde_json::Map::new();
        map.insert("name".into(), json!("it%27s"));
        url_decode_map_values(&mut map).unwrap();
        assert_eq!(map["name"], json!("it's"));
    }

    #[test]
    fn test_url_decode_map_values_non_string_unchanged() {
        let mut map = serde_json::Map::new();
        map.insert("count".into(), json!(42));
        map.insert("active".into(), json!(true));
        url_decode_map_values(&mut map).unwrap();
        assert_eq!(map["count"], json!(42));
        assert_eq!(map["active"], json!(true));
    }

    #[test]
    fn test_url_decode_map_values_plain_string_unchanged() {
        let mut map = serde_json::Map::new();
        map.insert("name".into(), json!("hello"));
        url_decode_map_values(&mut map).unwrap();
        assert_eq!(map["name"], json!("hello"));
    }

    // --- add_integer_column ---

    #[test]
    fn test_add_integer_column() {
        let mut cols = vec![];
        let mut phs = vec![];
        let mut args = serde_json::Map::new();
        let mut vals = serde_json::Map::new();
        add_integer_column(&mut cols, &mut phs, &mut args, &mut vals, "level", 5);
        assert_eq!(cols, vec!["\"level\""]);
        assert_eq!(phs, vec!["@p_level"]);
        assert_eq!(args["p_level"], json!({"type": "integer"}));
        assert_eq!(vals["p_level"], json!(5));
    }
}
