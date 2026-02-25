use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Map, Value, json};

use crate::easing::{MAX_LEVEL, generate_level_up_path_json};
use crate::encoding::url_decode;
use crate::error::AppError;
use crate::models;

use super::helpers::json_value_to_sql;

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
        let show_id = data
            .get("show_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidParameter("show_id is required".into()))?;
        let song_id = data
            .get("song_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::InvalidParameter("song_id is required".into()))?;
        let media_url = data.get("media_url").and_then(|v| v.as_str()).unwrap_or("");

        conn.execute(
            "INSERT INTO rel_show_song (show_id, song_id, media_url, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![show_id, song_id, media_url, now],
        )?;

        return Ok(json!({"id": format!("{show_id}:{song_id}")}));
    }

    let id = uuid::Uuid::new_v4().to_string();

    // Build columns and values
    let mut columns: Vec<&str> = vec!["id"];
    let mut placeholders: Vec<String> = vec!["?1".into()];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(id.clone())];
    let mut idx = 2u32;

    for key in allowed {
        if let Some(val) = data.get(*key) {
            columns.push(key);
            placeholders.push(format!("?{idx}"));
            values.push(json_value_to_sql(val));
            idx += 1;
        }
    }

    // Auto-add timestamps
    if table != "rel_show_song" {
        // created_at
        if !columns.contains(&"created_at") {
            columns.push("created_at");
            placeholders.push(format!("?{idx}"));
            values.push(Box::new(now));
            idx += 1;
        }
        // updated_at (for tables that have it)
        if matches!(table, "artist" | "show" | "song" | "learning")
            && !columns.contains(&"updated_at")
        {
            columns.push("updated_at");
            placeholders.push(format!("?{idx}"));
            values.push(Box::new(now));
            idx += 1;
        }
        // learning defaults
        if table == "learning" {
            if !columns.contains(&"level") {
                columns.push("level");
                placeholders.push(format!("?{idx}"));
                values.push(Box::new(0i64));
                idx += 1;
            }
            if !columns.contains(&"last_level_up_at") {
                columns.push("last_level_up_at");
                placeholders.push(format!("?{idx}"));
                values.push(Box::new(0i64));
                idx += 1;
            }
            if !columns.contains(&"graduated") {
                columns.push("graduated");
                placeholders.push(format!("?{idx}"));
                values.push(Box::new(0i64));
                idx += 1;
            }
            if !columns.contains(&"level_up_path") {
                columns.push("level_up_path");
                placeholders.push(format!("?{idx}"));
                values.push(Box::new(generate_level_up_path_json(MAX_LEVEL)));
                // idx not needed further but keep consistent
            }
        }
    }

    let cols_sql = columns
        .iter()
        .map(|c| format!("\"{c}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders_sql = placeholders.join(", ");
    let sql = format!("INSERT INTO \"{table}\" ({cols_sql}) VALUES ({placeholders_sql})");

    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, params_refs.as_slice())?;

    Ok(json!({"id": id}))
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
    let mut set_parts: Vec<String> = Vec::new();
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut idx = 1u32;

    // Check if level is being changed (for learning table)
    let level_changed = table == "learning" && data.contains_key("level");

    for key in allowed {
        if let Some(val) = data.get(*key) {
            set_parts.push(format!("\"{key}\" = ?{idx}"));
            values.push(json_value_to_sql(val));
            idx += 1;
        }
    }

    // Auto-update updated_at for tables that have it
    if matches!(table, "artist" | "show" | "song" | "learning") {
        set_parts.push(format!("\"updated_at\" = ?{idx}"));
        values.push(Box::new(now));
        idx += 1;
    }

    // Auto-update last_level_up_at when level changes
    if level_changed {
        set_parts.push(format!("\"last_level_up_at\" = ?{idx}"));
        values.push(Box::new(now));
        idx += 1;
    }

    // WHERE id = ?
    values.push(Box::new(id.to_string()));
    let where_idx = idx;

    let set_sql = set_parts.join(", ");
    let sql = format!("UPDATE \"{table}\" SET {set_sql} WHERE id = ?{where_idx}");

    let params_refs: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows_affected = conn.execute(&sql, params_refs.as_slice())?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Record not found: {table}/{id}"
        )));
    }

    Ok(json!({"updated": true}))
}

// ---------------------------------------------------------------------------
// delete <table> <id>
// ---------------------------------------------------------------------------

pub fn cmd_delete(conn: &mut Connection, table: &str, id: &str) -> Result<Value, AppError> {
    models::validate_table(table, models::DELETE_TABLES)?;

    // Use a read query to check existence first, then execute delete via JankenSQLHub
    let query_json = json!({
        "check_exists": {
            "query": "SELECT id FROM #[table] WHERE id=@id",
            "returns": ["id"],
            "args": {
                "table": {"enum": ["artist", "song"]},
                "id": {}
            }
        },
        "delete_by_id": {
            "query": "DELETE FROM #[table] WHERE id=@id",
            "args": {
                "table": {"enum": ["artist", "song"]},
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
