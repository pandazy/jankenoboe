use jankensqlhub::QueryDefinitions;
use rusqlite::Connection;
use serde_json::{Value, json};

use crate::easing::{MAX_LEVEL, generate_level_up_path_json};
use crate::error::AppError;
use crate::models;

/// The shared WHERE clause for finding due-for-review learning records.
/// Uses `@offset` (integer) as a look-ahead in seconds.
/// When offset=0, the behavior is identical to comparing against "now".
const DUE_WHERE: &str = "\
    l.graduated = 0 \
    AND ( \
        (l.last_level_up_at > 0 AND l.level = 0 \
         AND (CAST(strftime('%s', 'now') AS INTEGER) + @offset) >= (l.last_level_up_at + 300)) \
        OR \
        (l.last_level_up_at = 0 AND l.level = 0 \
         AND (CAST(strftime('%s', 'now') AS INTEGER) + @offset) >= (l.updated_at + 300)) \
        OR \
        (l.level > 0 \
         AND (json_extract(l.level_up_path, '$[' || l.level || ']') * 86400 + l.last_level_up_at) \
             <= (CAST(strftime('%s', 'now') AS INTEGER) + @offset)) \
    )";

// ---------------------------------------------------------------------------
// learning-due --limit
// ---------------------------------------------------------------------------

pub fn cmd_learning_due(
    conn: &mut Connection,
    limit: u32,
    offset_seconds: u32,
) -> Result<Value, AppError> {
    let query_json = json!({
        "learning_due": {
            "query": format!(
                "SELECT l.id, l.song_id, s.name as song_name, l.level, \
                 COALESCE(json_extract(l.level_up_path, '$[' || l.level || ']'), 0) as wait_days \
                 FROM learning l \
                 JOIN song s ON l.song_id = s.id \
                 WHERE {DUE_WHERE} \
                 ORDER BY l.level DESC \
                 LIMIT @limit"
            ),
            "returns": ["id", "song_id", "song_name", "level", "wait_days"],
            "args": {
                "offset": {"type": "integer"},
                "limit": {"type": "integer"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let params = json!({
        "offset": offset_seconds,
        "limit": limit
    });

    let result = jankensqlhub::query_run_sqlite(conn, &queries, "learning_due", &params)
        .map_err(AppError::from)?;

    let count = result.data.len();
    Ok(json!({"count": count, "results": result.data}))
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

    let query_json = json!({
        "check_song_exists": {
            "query": "SELECT COUNT(*) as cnt FROM song WHERE id=@song_id",
            "returns": ["cnt"]
        },
        "count_active_learning": {
            "query": "SELECT COUNT(*) as cnt FROM learning WHERE song_id=@song_id AND graduated=0",
            "returns": ["cnt"]
        },
        "count_graduated_learning": {
            "query": "SELECT COUNT(*) as cnt FROM learning WHERE song_id=@song_id AND graduated=1",
            "returns": ["cnt"]
        },
        "insert_learning": {
            "query": "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) \
                      VALUES (@id, @song_id, @level, @now, @now, @last_level_up_at, @level_up_path, 0)",
            "args": {
                "level": {"type": "integer"},
                "now": {"type": "integer"},
                "last_level_up_at": {"type": "integer"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let mut created_ids: Vec<String> = Vec::new();
    let mut skipped_song_ids: Vec<String> = Vec::new();
    let mut already_graduated_song_ids: Vec<String> = Vec::new();

    let tx = conn.transaction()?;

    for song_id in &song_ids {
        // Verify song exists
        let song_params = json!({"song_id": song_id});
        let exists_result = jankensqlhub::query_run_sqlite_with_transaction(
            &tx,
            &queries,
            "check_song_exists",
            &song_params,
        )
        .map_err(AppError::from)?;

        let song_count = exists_result.data[0]["cnt"].as_i64().unwrap_or(0);
        if song_count == 0 {
            tx.rollback().ok();
            return Err(AppError::InvalidParameter(format!(
                "song not found: {song_id}"
            )));
        }

        // Check existing active learning records
        let active_result = jankensqlhub::query_run_sqlite_with_transaction(
            &tx,
            &queries,
            "count_active_learning",
            &song_params,
        )
        .map_err(AppError::from)?;

        let active_count = active_result.data[0]["cnt"].as_i64().unwrap_or(0);
        if active_count > 0 {
            skipped_song_ids.push(song_id.to_string());
            continue;
        }

        // Check graduated learning records
        let grad_result = jankensqlhub::query_run_sqlite_with_transaction(
            &tx,
            &queries,
            "count_graduated_learning",
            &song_params,
        )
        .map_err(AppError::from)?;

        let graduated_count = grad_result.data[0]["cnt"].as_i64().unwrap_or(0);

        if graduated_count > 0 {
            if relearn_ids.contains(song_id) {
                // Re-learn: create new record at relearn_start_level
                let new_id = uuid::Uuid::new_v4().to_string();
                let insert_params = json!({
                    "id": new_id,
                    "song_id": song_id,
                    "level": relearn_start_level,
                    "now": now,
                    "last_level_up_at": now,
                    "level_up_path": level_up_path
                });
                jankensqlhub::query_run_sqlite_with_transaction(
                    &tx,
                    &queries,
                    "insert_learning",
                    &insert_params,
                )
                .map_err(AppError::from)?;
                created_ids.push(new_id);
            } else {
                already_graduated_song_ids.push(song_id.to_string());
            }
            continue;
        }

        // No existing record - create new
        let new_id = uuid::Uuid::new_v4().to_string();
        let insert_params = json!({
            "id": new_id,
            "song_id": song_id,
            "level": 0,
            "now": now,
            "last_level_up_at": 0,
            "level_up_path": level_up_path
        });
        jankensqlhub::query_run_sqlite_with_transaction(
            &tx,
            &queries,
            "insert_learning",
            &insert_params,
        )
        .map_err(AppError::from)?;
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
    // Step 1: Get due songs (reuses shared DUE_WHERE)
    let query_json = json!({
        "due_songs": {
            "query": format!(
                "SELECT l.id, l.song_id, s.name as song_name, l.level, \
                 COALESCE(json_extract(l.level_up_path, '$[' || l.level || ']'), 0) as wait_days, \
                 s.artist_id \
                 FROM learning l \
                 JOIN song s ON l.song_id = s.id \
                 WHERE {DUE_WHERE} \
                 ORDER BY l.level DESC \
                 LIMIT @limit"
            ),
            "returns": ["id", "song_id", "song_name", "level", "wait_days", "artist_id"],
            "args": {
                "offset": {"type": "integer"},
                "limit": {"type": "integer"}
            }
        },
        "get_artist_name": {
            "query": "SELECT name FROM artist WHERE id=@artist_id",
            "returns": ["name"]
        },
        "get_show_info": {
            "query": "SELECT rs.show_id, rs.media_url, sh.name as show_name \
                      FROM rel_show_song rs \
                      JOIN show sh ON rs.show_id = sh.id \
                      WHERE rs.song_id=@song_id",
            "returns": ["show_id", "media_url", "show_name"]
        },
        "get_play_history_urls": {
            "query": "SELECT media_url FROM play_history WHERE song_id=@song_id AND media_url != '' AND status=0",
            "returns": ["media_url"]
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    let due_params = json!({
        "offset": offset_seconds,
        "limit": limit
    });

    let due_result = jankensqlhub::query_run_sqlite(conn, &queries, "due_songs", &due_params)
        .map_err(AppError::from)?;

    let count = due_result.data.len();

    // Step 2: Enrich each song
    let mut songs: Vec<EnrichedSong> = Vec::new();

    for row in &due_result.data {
        let song_id = row["song_id"].as_str().unwrap_or("");
        let song_name = row["song_name"].as_str().unwrap_or("");
        let level = row["level"].as_i64().unwrap_or(0);
        let wait_days = row["wait_days"].as_i64().unwrap_or(0);
        let artist_id = row["artist_id"].as_str().unwrap_or("");

        // Get artist name
        let artist_params = json!({"artist_id": artist_id});
        let artist_result =
            jankensqlhub::query_run_sqlite(conn, &queries, "get_artist_name", &artist_params)
                .map_err(AppError::from)?;
        let artist_name = artist_result
            .data
            .first()
            .and_then(|r| r["name"].as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Get show names and media URLs from rel_show_song
        let song_params = json!({"song_id": song_id});
        let show_result =
            jankensqlhub::query_run_sqlite(conn, &queries, "get_show_info", &song_params)
                .map_err(AppError::from)?;

        let mut show_names: Vec<String> = Vec::new();
        let mut media_urls: Vec<String> = Vec::new();

        for show_row in &show_result.data {
            let show_name = show_row["show_name"].as_str().unwrap_or("").to_string();
            if !show_name.is_empty() && !show_names.contains(&show_name) {
                show_names.push(show_name);
            }
            if let Some(url) = show_row["media_url"].as_str()
                && !url.is_empty()
                && !media_urls.contains(&url.to_string())
            {
                media_urls.push(url.to_string());
            }
        }

        // Get media URLs from play_history
        let ph_result =
            jankensqlhub::query_run_sqlite(conn, &queries, "get_play_history_urls", &song_params)
                .map_err(AppError::from)?;

        for ph_row in &ph_result.data {
            if let Some(url) = ph_row["media_url"].as_str()
                && !url.is_empty()
                && !media_urls.contains(&url.to_string())
            {
                media_urls.push(url.to_string());
            }
        }

        songs.push(EnrichedSong {
            song_name: song_name.to_string(),
            level,
            wait_days,
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
    let learning_ids: Vec<&str> = due_result
        .data
        .iter()
        .filter_map(|row| row["id"].as_str())
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

    let query_json = json!({
        "get_learning_record": {
            "query": "SELECT id, level, graduated FROM learning WHERE id=@id",
            "returns": ["id", "level", "graduated"],
            "args": {
                "id": {}
            }
        },
        "level_up": {
            "query": "UPDATE learning SET level=@new_level, updated_at=@now, last_level_up_at=@now WHERE id=@id",
            "args": {
                "new_level": {"type": "integer"},
                "now": {"type": "integer"}
            }
        },
        "graduate": {
            "query": "UPDATE learning SET graduated=1, updated_at=@now, last_level_up_at=@now WHERE id=@id",
            "args": {
                "now": {"type": "integer"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Query definition error: {e}")))?;

    // Fetch current level for each ID, verify they exist and are not graduated
    let mut records: Vec<(String, i64)> = Vec::new();
    let mut not_found_ids: Vec<String> = Vec::new();

    for id in &ids {
        let params = json!({"id": id});
        let result = jankensqlhub::query_run_sqlite(conn, &queries, "get_learning_record", &params)
            .map_err(AppError::from)?;

        if result.data.is_empty() {
            not_found_ids.push(id.to_string());
            continue;
        }

        let row = &result.data[0];
        let record_id = row["id"].as_str().unwrap_or("").to_string();
        let level = row["level"].as_i64().unwrap_or(0);
        let graduated = row["graduated"].as_i64().unwrap_or(0);

        if graduated == 1 {
            return Err(AppError::InvalidParameter(format!(
                "learning record already graduated: {record_id}"
            )));
        }
        records.push((record_id, level));
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
            let params = json!({"id": id, "now": now});
            jankensqlhub::query_run_sqlite_with_transaction(&tx, &queries, "graduate", &params)
                .map_err(AppError::from)?;
            graduated_count += 1;
        } else {
            // Level up by 1
            let new_level = level + 1;
            let params = json!({"id": id, "new_level": new_level, "now": now});
            jankensqlhub::query_run_sqlite_with_transaction(&tx, &queries, "level_up", &params)
                .map_err(AppError::from)?;
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

/// Build the final HTML by loading the template and replacing placeholders.
fn build_review_html(
    songs: &[impl SongReviewData],
    level_dist: &std::collections::BTreeMap<i64, usize>,
) -> String {
    let template = include_str!("../../templates/learning-song-review.html");

    let total = songs.len().to_string();

    // Build level distribution JSON array: [{level, count}, ...]
    let dist_json: Vec<Value> = level_dist
        .iter()
        .map(|(level, count)| json!({"level": level + 1, "count": count}))
        .collect();
    let dist_json_str = serde_json::to_string(&dist_json).unwrap_or_else(|_| "[]".to_string());

    // Build songs JSON array for client-side rendering
    let songs_data: Vec<Value> = songs
        .iter()
        .map(|s| {
            let shows_joined = s
                .show_names()
                .iter()
                .map(|n| escape_html(n))
                .collect::<Vec<_>>()
                .join(" | ");
            let media_urls: Vec<Value> = s
                .media_urls()
                .iter()
                .map(|url| {
                    let ext = extract_url_extension(url);
                    json!({"url": escape_html(url), "ext": ext})
                })
                .collect();
            json!({
                "name": escape_html(s.song_name()),
                "artist": escape_html(s.artist_name()),
                "level": s.level() + 1,
                "waitDays": s.wait_days(),
                "shows": shows_joined,
                "mediaUrls": media_urls
            })
        })
        .collect();
    let songs_json_str = serde_json::to_string(&songs_data).unwrap_or_else(|_| "[]".to_string());

    // Replace placeholders in the template
    template
        .replace("{{TOTAL}}", &total)
        .replace("{{DIST_JSON}}", &dist_json_str)
        .replace("{{SONGS_JSON}}", &songs_json_str)
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- escape_html ---

    #[test]
    fn test_escape_html_special_chars() {
        assert_eq!(escape_html("<b>bold</b>"), "&lt;b&gt;bold&lt;/b&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(escape_html("it's"), "it&#39;s");
    }

    #[test]
    fn test_escape_html_plain_text() {
        assert_eq!(escape_html("hello world"), "hello world");
        assert_eq!(escape_html(""), "");
    }

    // --- extract_url_extension ---

    #[test]
    fn test_extract_url_extension_mp3() {
        assert_eq!(
            extract_url_extension("https://example.com/song.mp3"),
            ".mp3"
        );
    }

    #[test]
    fn test_extract_url_extension_with_query() {
        assert_eq!(
            extract_url_extension("https://example.com/video.webm?t=123"),
            ".webm"
        );
    }

    #[test]
    fn test_extract_url_extension_with_fragment() {
        assert_eq!(
            extract_url_extension("https://example.com/file.wav#anchor"),
            ".wav"
        );
    }

    #[test]
    fn test_extract_url_extension_no_extension() {
        assert_eq!(extract_url_extension("https://example.com/noext"), "");
    }

    #[test]
    fn test_extract_url_extension_empty_url() {
        assert_eq!(extract_url_extension(""), "");
    }

    #[test]
    fn test_extract_url_extension_uppercase() {
        assert_eq!(
            extract_url_extension("https://example.com/song.MP3"),
            ".mp3"
        );
    }

    // --- build_review_html ---

    struct TestSong {
        name: String,
        level: i64,
        wait_days: i64,
        artist: String,
        shows: Vec<String>,
        urls: Vec<String>,
    }

    impl SongReviewData for TestSong {
        fn song_name(&self) -> &str {
            &self.name
        }
        fn level(&self) -> i64 {
            self.level
        }
        fn wait_days(&self) -> i64 {
            self.wait_days
        }
        fn artist_name(&self) -> &str {
            &self.artist
        }
        fn show_names(&self) -> &[String] {
            &self.shows
        }
        fn media_urls(&self) -> &[String] {
            &self.urls
        }
    }

    #[test]
    fn test_build_review_html_empty() {
        let songs: Vec<TestSong> = vec![];
        let dist = std::collections::BTreeMap::new();
        let html = build_review_html(&songs, &dist);
        assert!(html.contains("Total due: 0 songs"));
        assert!(html.contains("SONGS = []"));
        assert!(html.contains("LEVEL_DIST = []"));
    }

    #[test]
    fn test_build_review_html_with_songs() {
        let songs = vec![TestSong {
            name: "Test Song".into(),
            level: 5,
            wait_days: 3,
            artist: "Test Artist".into(),
            shows: vec!["Show A".into()],
            urls: vec!["https://example.com/video.webm".into()],
        }];
        let mut dist = std::collections::BTreeMap::new();
        dist.insert(5, 1);
        let html = build_review_html(&songs, &dist);
        assert!(html.contains("Total due: 1 songs"));
        assert!(html.contains("Test Song"));
        assert!(html.contains("Test Artist"));
        assert!(html.contains("Show A"));
        assert!(html.contains("https://example.com/video.webm"));
        assert!(html.contains(".webm"));
        // Level display: stored 5 → displayed 6
        assert!(html.contains("\"level\":6"));
    }

    #[test]
    fn test_build_review_html_no_media() {
        let songs = vec![TestSong {
            name: "No Media Song".into(),
            level: 0,
            wait_days: 1,
            artist: "Artist".into(),
            shows: vec![],
            urls: vec![],
        }];
        let mut dist = std::collections::BTreeMap::new();
        dist.insert(0, 1);
        let html = build_review_html(&songs, &dist);
        assert!(html.contains("\"mediaUrls\":[]"));
    }

    #[test]
    fn test_build_review_html_html_escaping() {
        let songs = vec![TestSong {
            name: "<script>alert('xss')</script>".into(),
            level: 0,
            wait_days: 1,
            artist: "O'Brien & Co".into(),
            shows: vec!["Show <1>".into()],
            urls: vec![],
        }];
        let mut dist = std::collections::BTreeMap::new();
        dist.insert(0, 1);
        let html = build_review_html(&songs, &dist);
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("O&#39;Brien &amp; Co"));
    }
}
