use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Map, Value, json};

use crate::encoding::url_decode;
use crate::error::AppError;
use crate::models;
use crate::table_config;

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
    let fields = models::parse_fields(fields_str);
    if fields.is_empty() {
        return Err(AppError::InvalidParameter("fields cannot be empty".into()));
    }

    let term: Map<String, Value> = serde_json::from_str(term_json)?;
    if term.is_empty() {
        return Err(AppError::InvalidParameter("term cannot be empty".into()));
    }

    // Validate term keys against searchable fields for this table
    let searchable = models::allowed_term_keys(table)?;
    let mut where_parts: Vec<String> = Vec::new();
    let mut val_args = serde_json::Map::new();
    let mut val_values = serde_json::Map::new();

    for (col, cond) in &term {
        // Validate column name against searchable whitelist
        if !searchable.contains(&col.as_str()) {
            return Err(AppError::InvalidParameter(format!(
                "Invalid term key for {table}: {col}. Allowed: {}",
                searchable.join(", ")
            )));
        }

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

        let val_key = format!("val_{col}");

        // Column name is safe to embed directly — validated against searchable whitelist above
        let prepared_value = match match_mode {
            "exact" => {
                where_parts.push(format!("\"{col}\"=@{val_key}"));
                value
            }
            "exact-i" => {
                where_parts.push(format!("LOWER(\"{col}\")=LOWER(@{val_key})"));
                value
            }
            "starts-with" => {
                where_parts.push(format!("LOWER(\"{col}\") LIKE LOWER(@{val_key})"));
                format!("{value}%")
            }
            "ends-with" => {
                where_parts.push(format!("LOWER(\"{col}\") LIKE LOWER(@{val_key})"));
                format!("%{value}")
            }
            "contains" => {
                where_parts.push(format!("LOWER(\"{col}\") LIKE LOWER(@{val_key})"));
                format!("%{value}%")
            }
            _ => unreachable!(),
        };

        val_args.insert(val_key.clone(), json!({}));
        val_values.insert(val_key, json!(prepared_value));
    }

    let where_sql = where_parts.join(" AND ");

    // Build args: table + fields + per-value params
    let mut args = json!({
        "table": {"enum": table_config::build_table_enum(models::SEARCH_TABLES)},
        "fields": {
            "enumif": table_config::build_selectable_enumif(models::SEARCH_TABLES)
        }
    });
    let args_map = args.as_object_mut().unwrap();
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

    // Build params: table + fields + search values
    let mut params = json!({
        "table": table,
        "fields": fields,
    });
    let params_map = params.as_object_mut().unwrap();
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

    let table_enum = table_config::build_table_enum(models::DUPLICATES_TABLES);

    // Two query variants: artist/song include a song_count subquery; show uses 0
    let (query_name, query_sql) = match table {
        "artist" | "song" => (
            "duplicates_with_song_count",
            "SELECT a.id, a.name, \
             (SELECT COUNT(*) FROM song s WHERE s.artist_id = a.id) as song_count \
             FROM #[table] a \
             WHERE LOWER(a.name) IN ( \
               SELECT LOWER(name) FROM #[table] \
               WHERE status = 0 \
               GROUP BY LOWER(name) HAVING COUNT(*) > 1 \
             ) AND a.status = 0 \
             ORDER BY LOWER(a.name), a.name",
        ),
        "show" => (
            "duplicates_no_song_count",
            "SELECT a.id, a.name, \
             0 as song_count \
             FROM #[table] a \
             WHERE LOWER(a.name) IN ( \
               SELECT LOWER(name) FROM #[table] \
               WHERE status = 0 \
               GROUP BY LOWER(name) HAVING COUNT(*) > 1 \
             ) AND a.status = 0 \
             ORDER BY LOWER(a.name), a.name",
        ),
        _ => unreachable!(),
    };

    let query_json = json!({
        query_name: {
            "query": query_sql,
            "returns": ["id", "name", "song_count"],
            "args": {
                "table": {"enum": table_enum}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!({ "table": table });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, query_name, &params)
        .map_err(AppError::from)?;

    // Group rows by lowercase name (rows are already ordered by LOWER(name))
    let mut groups: Vec<Value> = Vec::new();
    let mut current_group_name: Option<String> = None;
    let mut current_records: Vec<Value> = Vec::new();

    for row in &result.data {
        let name = row["name"].as_str().unwrap_or("");
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
            "id": row["id"],
            "name": row["name"],
            "song_count": row["song_count"]
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
// shows-by-artist-ids --artist-ids
// ---------------------------------------------------------------------------

pub fn cmd_shows_by_artist_ids(
    conn: &mut Connection,
    artist_ids_str: &str,
) -> Result<Value, AppError> {
    let artist_ids: Vec<&str> = artist_ids_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if artist_ids.is_empty() {
        return Err(AppError::InvalidParameter(
            "artist_ids cannot be empty".into(),
        ));
    }

    let query_json = json!({
        "shows_by_artists": {
            "query": "SELECT DISTINCT sh.id as show_id, sh.name as show_name, sh.vintage, \
                      s.id as song_id, s.name as song_name, \
                      a.id as artist_id, a.name as artist_name \
                      FROM show sh \
                      JOIN rel_show_song rs ON rs.show_id = sh.id \
                      JOIN song s ON rs.song_id = s.id \
                      JOIN artist a ON s.artist_id = a.id \
                      WHERE a.id IN :[artist_ids] \
                      ORDER BY a.name, sh.name, s.name",
            "returns": ["show_id", "show_name", "vintage", "song_id", "song_name", "artist_id", "artist_name"],
            "args": {
                "artist_ids": {"itemtype": "string"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let ids_json: Vec<Value> = artist_ids.iter().map(|s| json!(s)).collect();
    let params = json!({ "artist_ids": ids_json });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "shows_by_artists", &params)
        .map_err(AppError::from)?;

    let count = result.data.len();
    Ok(json!({"count": count, "results": result.data}))
}

// ---------------------------------------------------------------------------
// songs-by-artist-ids --artist-ids
// ---------------------------------------------------------------------------

pub fn cmd_songs_by_artist_ids(
    conn: &mut Connection,
    artist_ids_str: &str,
) -> Result<Value, AppError> {
    let artist_ids: Vec<&str> = artist_ids_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if artist_ids.is_empty() {
        return Err(AppError::InvalidParameter(
            "artist_ids cannot be empty".into(),
        ));
    }

    let query_json = json!({
        "songs_by_artists": {
            "query": "SELECT s.id as song_id, s.name as song_name, \
                      a.id as artist_id, a.name as artist_name \
                      FROM song s \
                      JOIN artist a ON s.artist_id = a.id \
                      WHERE a.id IN :[artist_ids] \
                      ORDER BY a.name, s.name",
            "returns": ["song_id", "song_name", "artist_id", "artist_name"],
            "args": {
                "artist_ids": {"itemtype": "string"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let ids_json: Vec<Value> = artist_ids.iter().map(|s| json!(s)).collect();
    let params = json!({ "artist_ids": ids_json });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "songs_by_artists", &params)
        .map_err(AppError::from)?;

    let count = result.data.len();
    Ok(json!({"count": count, "results": result.data}))
}
