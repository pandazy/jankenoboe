use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Map, Value, json};

use crate::easing::{MAX_LEVEL, generate_level_up_path_json};
use crate::encoding::url_decode;
use crate::error::AppError;
use crate::models;
use crate::table_config;

/// Build the SQL WHERE clause for finding due-for-review learning records.
/// The `offset_seconds` parameter shifts the reference time forward into the future,
/// allowing queries like "songs due in the next 2 hours" (offset_seconds = 7200).
/// When offset_seconds is 0, the behavior is identical to comparing against "now".
fn build_due_where(offset_seconds: u32) -> String {
    let now_expr = if offset_seconds == 0 {
        "CAST(strftime('%s', 'now') AS INTEGER)".to_string()
    } else {
        format!("(CAST(strftime('%s', 'now') AS INTEGER) + {offset_seconds})")
    };
    format!(
        "l.graduated = 0 \
         AND ( \
             (l.last_level_up_at > 0 AND l.level = 0 \
              AND {now_expr} >= (l.last_level_up_at + 300)) \
             OR \
             (l.last_level_up_at = 0 AND l.level = 0 \
              AND {now_expr} >= (l.updated_at + 300)) \
             OR \
             (l.level > 0 \
              AND (json_extract(l.level_up_path, '$[' || l.level || ']') * 86400 + l.last_level_up_at) \
                  <= {now_expr}) \
         )"
    )
}

// ---------------------------------------------------------------------------
// get <table> <id> --fields
// ---------------------------------------------------------------------------

pub fn cmd_get(
    conn: &mut Connection,
    table: &str,
    id: &str,
    fields_str: &str,
) -> Result<Value, AppError> {
    models::validate_table(table, models::GET_TABLES)?;
    let fields = models::parse_fields(fields_str);
    if fields.is_empty() {
        return Err(AppError::InvalidParameter("fields cannot be empty".into()));
    }
    let allowed = models::get_fields(table)?;
    models::validate_fields(&fields, allowed)?;

    let query_json = json!({
        "read_by_id": {
            "query": "SELECT ~[fields] FROM #[table] WHERE id=@id",
            "returns": "~[fields]",
            "args": {
                "table": {"enum": table_config::build_table_enum(models::GET_TABLES)},
                "fields": {
                    "enumif": table_config::build_selectable_enumif(models::GET_TABLES)
                }
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!({
        "table": table,
        "id": id,
        "fields": fields
    });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "read_by_id", &params)
        .map_err(AppError::from)?;

    Ok(json!({"results": result.data}))
}

// ---------------------------------------------------------------------------
// search <table> --term --fields
// ---------------------------------------------------------------------------

pub fn cmd_search(
    conn: &mut Connection,
    table: &str,
    term_json: &str,
    fields_str: &str,
) -> Result<Value, AppError> {
    models::validate_table(table, models::SEARCH_TABLES)?;
    let fields = models::parse_fields(fields_str);
    if fields.is_empty() {
        return Err(AppError::InvalidParameter("fields cannot be empty".into()));
    }

    let term: Map<String, Value> = serde_json::from_str(term_json)?;
    if term.is_empty() {
        return Err(AppError::InvalidParameter("term cannot be empty".into()));
    }

    // Parse term conditions: build WHERE parts, col args, value args
    let searchable_enumif = table_config::build_searchable_enumif(models::SEARCH_TABLES);
    let mut where_parts: Vec<String> = Vec::new();
    let mut col_args = serde_json::Map::new();
    let mut col_values = serde_json::Map::new();
    let mut val_args = serde_json::Map::new();
    let mut val_values = serde_json::Map::new();

    for (col, cond) in &term {
        let cond_obj = cond.as_object().ok_or_else(|| {
            AppError::InvalidParameter(format!("Term condition for '{col}' must be an object"))
        })?;
        let raw_value = cond_obj
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AppError::InvalidParameter(format!(
                    "Term condition for '{col}' must have a 'value' string"
                ))
            })?;
        let value = url_decode(raw_value).map_err(|e| {
            AppError::InvalidParameter(format!(
                "URL decoding error for search value of '{col}': {e}"
            ))
        })?;
        let match_mode = cond_obj
            .get("match")
            .and_then(|v| v.as_str())
            .unwrap_or("exact");

        if !models::MATCH_MODES.contains(&match_mode) {
            return Err(AppError::InvalidParameter(format!(
                "Invalid match mode: {match_mode}. Allowed: {}",
                models::MATCH_MODES.join(", ")
            )));
        }

        let col_key = format!("col_{col}");
        let val_key = format!("val_{col}");

        // Column is a #[col_X] identifier validated by shared searchable enumif
        col_args.insert(col_key.clone(), json!({"enumif": searchable_enumif}));
        col_values.insert(col_key.clone(), json!(col));

        let prepared_value = match match_mode {
            "exact" => {
                where_parts.push(format!("#[{col_key}]=@{val_key}"));
                value
            }
            "exact-i" => {
                where_parts.push(format!("LOWER(#[{col_key}])=LOWER(@{val_key})"));
                value
            }
            "starts-with" => {
                where_parts.push(format!("LOWER(#[{col_key}]) LIKE LOWER(@{val_key})"));
                format!("{value}%")
            }
            "ends-with" => {
                where_parts.push(format!("LOWER(#[{col_key}]) LIKE LOWER(@{val_key})"));
                format!("%{value}")
            }
            "contains" => {
                where_parts.push(format!("LOWER(#[{col_key}]) LIKE LOWER(@{val_key})"));
                format!("%{value}%")
            }
            _ => unreachable!(),
        };

        val_args.insert(val_key.clone(), json!({}));
        val_values.insert(val_key, json!(prepared_value));
    }

    let where_sql = where_parts.join(" AND ");

    // Build args: table + fields + per-column + per-value
    let mut args = json!({
        "table": {"enum": table_config::build_table_enum(models::SEARCH_TABLES)},
        "fields": {
            "enumif": table_config::build_selectable_enumif(models::SEARCH_TABLES)
        }
    });
    let args_map = args.as_object_mut().unwrap();
    for (k, v) in col_args {
        args_map.insert(k, v);
    }
    for (k, v) in val_args {
        args_map.insert(k, v);
    }

    let query_json = json!({
        "search": {
            "query": format!("SELECT ~[fields] FROM #[table] WHERE {where_sql}"),
            "returns": "~[fields]",
            "args": args
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    // Build params: table + fields + col values + search values
    let mut params = json!({
        "table": table,
        "fields": fields,
    });
    let params_map = params.as_object_mut().unwrap();
    for (k, v) in col_values {
        params_map.insert(k, v);
    }
    for (k, v) in val_values {
        params_map.insert(k, v);
    }

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "search", &params)
        .map_err(AppError::from)?;

    Ok(json!({"results": result.data}))
}

// ---------------------------------------------------------------------------
// duplicates <table>
// ---------------------------------------------------------------------------

pub fn cmd_duplicates(conn: &mut Connection, table: &str) -> Result<Value, AppError> {
    models::validate_table(table, models::DUPLICATES_TABLES)?;

    // Find groups with case-insensitive matching names
    let sql = match table {
        "artist" | "song" => format!(
            "SELECT a.id, a.name, \
             (SELECT COUNT(*) FROM song s WHERE s.artist_id = a.id) as song_count \
             FROM \"{table}\" a \
             WHERE LOWER(a.name) IN ( \
               SELECT LOWER(name) FROM \"{table}\" \
               WHERE status = 0 \
               GROUP BY LOWER(name) HAVING COUNT(*) > 1 \
             ) AND a.status = 0 \
             ORDER BY LOWER(a.name), a.name"
        ),
        "show" => "SELECT a.id, a.name, \
             0 as song_count \
             FROM \"show\" a \
             WHERE LOWER(a.name) IN ( \
               SELECT LOWER(name) FROM \"show\" \
               WHERE status = 0 \
               GROUP BY LOWER(name) HAVING COUNT(*) > 1 \
             ) AND a.status = 0 \
             ORDER BY LOWER(a.name), a.name"
            .to_string(),
        _ => unreachable!(),
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let song_count: i64 = row.get(2)?;
        Ok((id, name, song_count))
    })?;

    let mut all_records: Vec<(String, String, i64)> = Vec::new();
    for r in rows {
        all_records.push(r?);
    }

    // Group by lowercase name
    let mut groups: Vec<Value> = Vec::new();
    let mut current_group_name: Option<String> = None;
    let mut current_records: Vec<Value> = Vec::new();

    for (id, name, song_count) in &all_records {
        let lower = name.to_lowercase();
        if current_group_name.as_ref() != Some(&lower) {
            if !current_records.is_empty() {
                groups.push(json!({
                    "name": current_group_name.as_deref().unwrap_or(""),
                    "records": current_records
                }));
                current_records = Vec::new();
            }
            current_group_name = Some(lower);
        }
        current_records.push(json!({
            "id": id,
            "name": name,
            "song_count": song_count
        }));
    }
    if !current_records.is_empty() {
        groups.push(json!({
            "name": current_group_name.as_deref().unwrap_or(""),
            "records": current_records
        }));
    }

    Ok(json!({"duplicates": groups}))
}

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
// learning-due --limit
// ---------------------------------------------------------------------------

pub fn cmd_learning_due(
    conn: &mut Connection,
    limit: u32,
    offset_seconds: u32,
) -> Result<Value, AppError> {
    let due_where = build_due_where(offset_seconds);
    let sql = format!(
        "SELECT l.id, l.song_id, s.name as song_name, l.level, \
               json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days \
         FROM learning l \
         JOIN song s ON l.song_id = s.id \
         WHERE {due_where} \
         ORDER BY l.level DESC \
         LIMIT ?1"
    );
    let sql = sql.as_str();

    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![limit], |row| {
        let id: String = row.get(0)?;
        let song_id: String = row.get(1)?;
        let song_name: String = row.get(2)?;
        let level: i64 = row.get(3)?;
        let wait_days: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);
        Ok(json!({
            "id": id,
            "song_id": song_id,
            "song_name": song_name,
            "level": level,
            "wait_days": wait_days
        }))
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    let count = results.len();

    Ok(json!({"count": count, "results": results}))
}

// ---------------------------------------------------------------------------
// learning-batch --song-ids [--relearn-song-ids] [--relearn-start-level]
// ---------------------------------------------------------------------------

pub fn cmd_learning_batch(
    conn: &mut Connection,
    song_ids_str: &str,
    relearn_song_ids_str: Option<&str>,
    relearn_start_level: u32,
) -> Result<Value, AppError> {
    let song_ids: Vec<&str> = song_ids_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if song_ids.is_empty() {
        return Err(AppError::InvalidParameter(
            "song_ids cannot be empty".into(),
        ));
    }

    let relearn_ids: Vec<&str> = relearn_song_ids_str
        .map(|s| {
            s.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let now = models::now_unix();
    let level_up_path = generate_level_up_path_json(MAX_LEVEL);

    let mut created_ids: Vec<String> = Vec::new();
    let mut skipped_song_ids: Vec<String> = Vec::new();
    let mut already_graduated_song_ids: Vec<String> = Vec::new();

    let tx = conn.transaction()?;

    for song_id in &song_ids {
        // Verify song exists
        let song_exists: bool = tx
            .query_row(
                "SELECT COUNT(*) FROM song WHERE id = ?1",
                rusqlite::params![song_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)?;

        if !song_exists {
            tx.rollback().ok();
            return Err(AppError::InvalidParameter(format!(
                "song not found: {song_id}"
            )));
        }

        // Check existing learning records
        let active_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM learning WHERE song_id = ?1 AND graduated = 0",
            rusqlite::params![song_id],
            |row| row.get(0),
        )?;

        if active_count > 0 {
            skipped_song_ids.push(song_id.to_string());
            continue;
        }

        let graduated_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM learning WHERE song_id = ?1 AND graduated = 1",
            rusqlite::params![song_id],
            |row| row.get(0),
        )?;

        if graduated_count > 0 {
            if relearn_ids.contains(song_id) {
                // Re-learn: create new record at relearn_start_level
                let new_id = uuid::Uuid::new_v4().to_string();
                tx.execute(
                    "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        new_id,
                        song_id,
                        relearn_start_level as i64,
                        now,
                        now,
                        now,
                        level_up_path,
                        0i64
                    ],
                )?;
                created_ids.push(new_id);
            } else {
                already_graduated_song_ids.push(song_id.to_string());
            }
            continue;
        }

        // No existing record - create new
        let new_id = uuid::Uuid::new_v4().to_string();
        tx.execute(
            "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![new_id, song_id, 0i64, now, now, 0i64, level_up_path, 0i64],
        )?;
        created_ids.push(new_id);
    }

    tx.commit()?;

    Ok(json!({
        "created_ids": created_ids,
        "skipped_song_ids": skipped_song_ids,
        "already_graduated_song_ids": already_graduated_song_ids
    }))
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
// learning-song-review --output --limit
// ---------------------------------------------------------------------------

pub fn cmd_learning_song_review(
    conn: &mut Connection,
    output_path: &str,
    limit: u32,
    offset_seconds: u32,
) -> Result<Value, AppError> {
    // Step 1: Get due songs (reuses shared build_due_where)
    let due_where = build_due_where(offset_seconds);
    let due_sql = format!(
        "SELECT l.id, l.song_id, s.name as song_name, l.level, \
               json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days, \
               s.artist_id \
         FROM learning l \
         JOIN song s ON l.song_id = s.id \
         WHERE {due_where} \
         ORDER BY l.level DESC \
         LIMIT ?1"
    );

    let mut stmt = conn.prepare(&due_sql)?;
    let due_rows = stmt.query_map(rusqlite::params![limit], |row| {
        let id: String = row.get(0)?;
        let song_id: String = row.get(1)?;
        let song_name: String = row.get(2)?;
        let level: i64 = row.get(3)?;
        let wait_days: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);
        let artist_id: String = row.get(5)?;
        Ok((id, song_id, song_name, level, wait_days, artist_id))
    })?;

    let mut due_songs_raw: Vec<(String, String, String, i64, i64, String)> = Vec::new();
    for r in due_rows {
        due_songs_raw.push(r?);
    }

    let count = due_songs_raw.len();

    // Step 2: Enrich each song (EnrichedSong implements SongReviewData below)

    let mut songs: Vec<EnrichedSong> = Vec::new();

    for (_id, song_id, song_name, level, wait_days, artist_id) in &due_songs_raw {
        // Get artist name
        let artist_name: String = conn
            .query_row(
                "SELECT name FROM artist WHERE id = ?1",
                rusqlite::params![artist_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get show names and media URLs from rel_show_song
        let mut show_names: Vec<String> = Vec::new();
        let mut media_urls: Vec<String> = Vec::new();

        {
            let mut rel_stmt = conn.prepare(
                "SELECT rs.show_id, rs.media_url, sh.name \
                 FROM rel_show_song rs \
                 JOIN show sh ON rs.show_id = sh.id \
                 WHERE rs.song_id = ?1",
            )?;
            let rel_rows = rel_stmt.query_map(rusqlite::params![song_id], |row| {
                let _show_id: String = row.get(0)?;
                let media_url: Option<String> = row.get(1)?;
                let show_name: String = row.get(2)?;
                Ok((show_name, media_url))
            })?;
            for r in rel_rows {
                let (show_name, media_url) = r?;
                if !show_names.contains(&show_name) {
                    show_names.push(show_name);
                }
                if let Some(url) = media_url
                    && !url.is_empty()
                    && !media_urls.contains(&url)
                {
                    media_urls.push(url);
                }
            }
        }

        // Get media URLs from play_history
        {
            let mut ph_stmt = conn.prepare(
                "SELECT media_url FROM play_history WHERE song_id = ?1 AND media_url != '' AND status = 0",
            )?;
            let ph_rows = ph_stmt.query_map(rusqlite::params![song_id], |row| {
                let url: String = row.get(0)?;
                Ok(url)
            })?;
            for r in ph_rows {
                let url = r?;
                if !url.is_empty() && !media_urls.contains(&url) {
                    media_urls.push(url);
                }
            }
        }

        songs.push(EnrichedSong {
            song_name: song_name.clone(),
            level: *level,
            wait_days: *wait_days,
            artist_name,
            show_names,
            media_urls,
        });
    }

    // Step 3: Compute level distribution
    let mut level_dist: std::collections::BTreeMap<i64, usize> = std::collections::BTreeMap::new();
    for s in &songs {
        *level_dist.entry(s.level).or_insert(0) += 1;
    }

    // Step 4: Generate HTML from template
    let html = build_review_html(&songs, &level_dist);

    // Step 5: Write to file
    let abs_path = if std::path::Path::new(output_path).is_absolute() {
        output_path.to_string()
    } else {
        std::env::current_dir()
            .map(|p| p.join(output_path).to_string_lossy().to_string())
            .unwrap_or_else(|_| output_path.to_string())
    };

    // Create parent directories if needed
    if let Some(parent) = std::path::Path::new(&abs_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Internal(format!("Failed to create directory: {e}")))?;
    }

    std::fs::write(&abs_path, html)
        .map_err(|e| AppError::Internal(format!("Failed to write HTML file: {e}")))?;

    // Collect learning IDs for use with learning-song-levelup-ids
    let learning_ids: Vec<&str> = due_songs_raw
        .iter()
        .map(|(id, _, _, _, _, _)| id.as_str())
        .collect();

    Ok(json!({
        "file": abs_path,
        "count": count,
        "learning_ids": learning_ids
    }))
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escape a string for embedding inside a JSON string literal.
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Build the final HTML by loading the template and replacing placeholders.
fn build_review_html(
    songs: &[impl SongReviewData],
    level_dist: &std::collections::BTreeMap<i64, usize>,
) -> String {
    let template = include_str!("../templates/learning-song-review.html");

    let total = songs.len().to_string();

    // Build level distribution HTML fragment
    let mut dist_html = String::new();
    for (level, count) in level_dist {
        dist_html.push_str(&format!(
            "<span class=\"level-badge\">Lv.{} × {}</span> ",
            level + 1,
            count
        ));
    }

    // Build songs JSON array for client-side pagination
    let mut songs_json = String::from("[");
    for (i, s) in songs.iter().enumerate() {
        if i > 0 {
            songs_json.push(',');
        }
        let shows_joined = s
            .show_names()
            .iter()
            .map(|n| escape_html(n))
            .collect::<Vec<_>>()
            .join(" | ");
        let urls_html: String = s
            .media_urls()
            .iter()
            .enumerate()
            .map(|(j, url)| {
                format!(
                    "<a href=\"{}\" target=\"_blank\" rel=\"noopener\">Media {}</a>",
                    escape_html(url),
                    j + 1
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let no_urls = if s.media_urls().is_empty() {
            "<span class=\"no-media\">No media URLs</span>"
        } else {
            ""
        };

        songs_json.push_str(&format!(
            "{{\"name\":\"{}\",\"artist\":\"{}\",\"level\":{},\"waitDays\":{},\"shows\":\"{}\",\"mediaHtml\":\"{}{}\"}}",
            escape_json_string(&escape_html(s.song_name())),
            escape_json_string(&escape_html(s.artist_name())),
            s.level() + 1,
            s.wait_days(),
            escape_json_string(&shows_joined),
            escape_json_string(&urls_html),
            escape_json_string(no_urls),
        ));
    }
    songs_json.push(']');

    // Replace placeholders in the template
    template
        .replace("{{TOTAL}}", &total)
        .replace("{{DIST_HTML}}", &dist_html)
        .replace("{{SONGS_JSON}}", &songs_json)
}

/// Data fields for a song in the review report.
struct EnrichedSong {
    song_name: String,
    level: i64,
    wait_days: i64,
    artist_name: String,
    show_names: Vec<String>,
    media_urls: Vec<String>,
}

/// Trait for accessing song review data fields (enables testability).
trait SongReviewData {
    fn song_name(&self) -> &str;
    fn level(&self) -> i64;
    fn wait_days(&self) -> i64;
    fn artist_name(&self) -> &str;
    fn show_names(&self) -> &[String];
    fn media_urls(&self) -> &[String];
}

impl SongReviewData for EnrichedSong {
    fn song_name(&self) -> &str {
        &self.song_name
    }
    fn level(&self) -> i64 {
        self.level
    }
    fn wait_days(&self) -> i64 {
        self.wait_days
    }
    fn artist_name(&self) -> &str {
        &self.artist_name
    }
    fn show_names(&self) -> &[String] {
        &self.show_names
    }
    fn media_urls(&self) -> &[String] {
        &self.media_urls
    }
}

// ---------------------------------------------------------------------------
// learning-song-levelup-ids --ids
// ---------------------------------------------------------------------------

pub fn cmd_learning_song_levelup_ids(
    conn: &mut Connection,
    ids_str: &str,
) -> Result<Value, AppError> {
    let ids: Vec<&str> = ids_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if ids.is_empty() {
        return Err(AppError::InvalidParameter("ids cannot be empty".into()));
    }

    // Fetch current level for each ID, verify they exist and are not graduated
    let mut records: Vec<(String, i64)> = Vec::new();
    let mut not_found_ids: Vec<String> = Vec::new();

    for id in &ids {
        let result = conn.query_row(
            "SELECT id, level, graduated FROM learning WHERE id = ?1",
            rusqlite::params![id],
            |row| {
                let id: String = row.get(0)?;
                let level: i64 = row.get(1)?;
                let graduated: i64 = row.get(2)?;
                Ok((id, level, graduated))
            },
        );

        match result {
            Ok((id, level, graduated)) => {
                if graduated == 1 {
                    return Err(AppError::InvalidParameter(format!(
                        "learning record already graduated: {id}"
                    )));
                }
                records.push((id, level));
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                not_found_ids.push(id.to_string());
            }
            Err(e) => return Err(AppError::Internal(e.to_string())),
        }
    }

    if !not_found_ids.is_empty() {
        return Err(AppError::NotFound(format!(
            "learning record(s) not found: {}",
            not_found_ids.join(", ")
        )));
    }

    let now = models::now_unix();
    let mut leveled_up_count: u64 = 0;
    let mut graduated_count: u64 = 0;

    let tx = conn.transaction()?;

    for (id, level) in &records {
        if *level >= (MAX_LEVEL as i64 - 1) {
            // Graduate
            tx.execute(
                "UPDATE learning SET graduated = 1, updated_at = ?1, last_level_up_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id],
            )?;
            graduated_count += 1;
        } else {
            // Level up by 1
            let new_level = level + 1;
            tx.execute(
                "UPDATE learning SET level = ?1, updated_at = ?2, last_level_up_at = ?2 WHERE id = ?3",
                rusqlite::params![new_level, now, id],
            )?;
            leveled_up_count += 1;
        }
    }

    tx.commit()?;

    Ok(json!({
        "leveled_up_count": leveled_up_count,
        "graduated_count": graduated_count,
        "total_processed": leveled_up_count + graduated_count
    }))
}

// ---------------------------------------------------------------------------
// Helpers
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

/// Convert a serde_json::Value to a boxed rusqlite::ToSql.
fn json_value_to_sql(val: &Value) -> Box<dyn rusqlite::ToSql> {
    match val {
        Value::String(s) => Box::new(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else if let Some(f) = n.as_f64() {
                Box::new(f)
            } else {
                Box::new(n.to_string())
            }
        }
        Value::Bool(b) => Box::new(*b as i64),
        Value::Null => Box::new(rusqlite::types::Null),
        _ => Box::new(val.to_string()),
    }
}
