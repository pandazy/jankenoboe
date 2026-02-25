use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Value, json};

use crate::easing::{MAX_LEVEL, generate_level_up_path_json};
use crate::error::AppError;
use crate::models;

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

    // Step 2: Enrich each song

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
// learning-by-song-ids --song-ids
// ---------------------------------------------------------------------------

pub fn cmd_learning_by_song_ids(
    conn: &mut Connection,
    song_ids_str: &str,
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

    let query_json = json!({
        "learning_by_songs": {
            "query": "SELECT l.id, l.song_id, s.name as song_name, l.level, l.graduated, \
                      l.last_level_up_at, \
                      json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days \
                      FROM learning l \
                      JOIN song s ON l.song_id = s.id \
                      WHERE l.song_id IN :[song_ids] \
                      ORDER BY l.level DESC",
            "returns": ["id", "song_id", "song_name", "level", "graduated", "last_level_up_at", "wait_days"],
            "args": {
                "song_ids": {"itemtype": "string"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let ids_json: Vec<Value> = song_ids.iter().map(|s| json!(s)).collect();
    let params = json!({ "song_ids": ids_json });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "learning_by_songs", &params)
        .map_err(AppError::from)?;

    let count = result.data.len();
    Ok(json!({"count": count, "results": result.data}))
}

// ---------------------------------------------------------------------------
// Review HTML helpers
// ---------------------------------------------------------------------------

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
    let template = include_str!("../../templates/learning-song-review.html");

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
                let ext = extract_url_extension(url);
                let label = if ext.is_empty() {
                    format!("Media {}", j + 1)
                } else {
                    format!("Media {} ({ext})", j + 1)
                };
                format!(
                    "<a href=\"{}\" target=\"_blank\" rel=\"noopener\">{}</a>",
                    escape_html(url),
                    label
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

/// Extract the file extension from a URL (e.g., ".mp3", ".webm").
/// Returns an empty string if no extension is found.
/// Strips query strings and fragments before extracting.
fn extract_url_extension(url: &str) -> String {
    // Remove query string and fragment
    let path = url.split('?').next().unwrap_or(url);
    let path = path.split('#').next().unwrap_or(path);

    // Find the last dot in the last path segment
    if let Some(last_segment) = path.rsplit('/').next()
        && let Some(dot_pos) = last_segment.rfind('.')
    {
        return last_segment[dot_pos..].to_lowercase();
    }
    String::new()
}
