use jankenoboe::commands;
use rusqlite::Connection;

fn test_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    conn.execute_batch(include_str!("../docs/init-db.sql"))
        .unwrap();
    conn
}

fn insert_artist(conn: &mut Connection, name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    conn.execute(
        "INSERT INTO artist (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, name, now, now],
    )
    .unwrap();
    id
}

fn insert_song(conn: &mut Connection, name: &str, artist_id: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    conn.execute(
        "INSERT INTO song (id, name, artist_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, artist_id, now, now],
    ).unwrap();
    id
}

fn insert_learning_raw(
    conn: &mut Connection,
    song_id: &str,
    level: i64,
    created_at: i64,
    updated_at: i64,
    last_up: i64,
    grad: i64,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let path = "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]";
    conn.execute(
        "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, song_id, level, created_at, updated_at, last_up, path, grad],
    ).unwrap();
    id
}

// === LEARNING-DUE ===

#[test]
fn test_learning_due_level0_newly_created() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 0, last_level_up_at=0, updated_at in the past (>300s ago)
    let past = jankenoboe::models::now_unix() - 400;
    let lid = insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 1);
    assert_eq!(r["results"][0]["id"], lid);
    assert_eq!(r["results"][0]["song_name"], "S");
    assert_eq!(r["results"][0]["level"], 0);
}

#[test]
fn test_learning_due_level0_with_last_level_up_at() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 0, last_level_up_at set to past (>300s ago)
    let past = jankenoboe::models::now_unix() - 400;
    insert_learning_raw(&mut c, &sid, 0, past, past, past, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 1);
}

#[test]
fn test_learning_due_level0_not_yet_due() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 0, updated_at = now (< 300s ago)
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 0, now, now, 0, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 0);
}

#[test]
fn test_learning_due_higher_level() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 1 with wait_days=1. last_level_up_at far in the past (>1 day ago)
    let past = jankenoboe::models::now_unix() - 90000; // >1 day
    insert_learning_raw(&mut c, &sid, 1, past, past, past, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 1);
    assert_eq!(r["results"][0]["wait_days"], 1);
}

#[test]
fn test_learning_due_higher_level_not_yet() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 7 with wait_days=2. last_level_up_at = now (not 2 days ago yet)
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 7, now, now, now, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 0);
}

#[test]
fn test_learning_due_graduated_excluded() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    insert_learning_raw(&mut c, &sid, 0, past, past, 0, 1); // graduated
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 0);
}

#[test]
fn test_learning_due_ordered_by_level_desc() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    // Level 10 wait_days=13, so need >13 days = >1123200 seconds
    let past = jankenoboe::models::now_unix() - 1200000;
    insert_learning_raw(&mut c, &s1, 3, past, past, past, 0);
    insert_learning_raw(&mut c, &s2, 10, past, past, past, 0);
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 2);
    // Higher level first
    assert_eq!(r["results"][0]["level"], 10);
    assert_eq!(r["results"][1]["level"], 3);
}

#[test]
fn test_learning_due_limit() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let past = jankenoboe::models::now_unix() - 400;
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    let s3 = insert_song(&mut c, "S3", &aid);
    insert_learning_raw(&mut c, &s1, 0, past, past, 0, 0);
    insert_learning_raw(&mut c, &s2, 0, past, past, 0, 0);
    insert_learning_raw(&mut c, &s3, 0, past, past, 0, 0);
    let r = commands::cmd_learning_due(&mut c, 2, 0).unwrap();
    assert_eq!(r["count"], 2);
}

// === LEARNING-DUE WITH OFFSET ===

#[test]
fn test_learning_due_offset_makes_not_yet_due_visible() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 0 created just now — not due without offset (needs 300s warm-up)
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 0, now, now, 0, 0);
    // Without offset: not due
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 0);
    // With 400s offset: now due (300s warm-up satisfied)
    let r = commands::cmd_learning_due(&mut c, 100, 400).unwrap();
    assert_eq!(r["count"], 1);
}

#[test]
fn test_learning_due_offset_higher_level() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Level 1, wait_days=1 (86400s). last_level_up_at = now — not due yet
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 1, now, now, now, 0);
    // Without offset: not due
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 0);
    // With offset of 2 days (172800s): due
    let r = commands::cmd_learning_due(&mut c, 100, 172800).unwrap();
    assert_eq!(r["count"], 1);
}

#[test]
fn test_learning_due_offset_zero_same_as_default() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);
    // offset=0 should behave identically to default
    let r = commands::cmd_learning_due(&mut c, 100, 0).unwrap();
    assert_eq!(r["count"], 1);
}

// === LEARNING-BATCH ===

#[test]
fn test_learning_batch_new_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let r = commands::cmd_learning_batch(&mut c, &sid, None, 7).unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["skipped_song_ids"].as_array().unwrap().len(), 0);
    assert_eq!(r["already_graduated_song_ids"].as_array().unwrap().len(), 0);
    // Verify record created
    let lid = r["created_ids"][0].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "learning", lid, "level,graduated,last_level_up_at").unwrap();
    assert_eq!(g["results"][0]["level"], 0);
    assert_eq!(g["results"][0]["graduated"], 0);
    assert_eq!(g["results"][0]["last_level_up_at"], 0);
}

#[test]
fn test_learning_batch_multiple_songs() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    let r = commands::cmd_learning_batch(&mut c, &format!("{s1},{s2}"), None, 7).unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 2);
}

#[test]
fn test_learning_batch_skip_active() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 5, now, now, now, 0); // active record
    let r = commands::cmd_learning_batch(&mut c, &sid, None, 7).unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 0);
    assert_eq!(r["skipped_song_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["skipped_song_ids"][0], sid);
}

#[test]
fn test_learning_batch_graduated_without_relearn() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 19, now, now, now, 1); // graduated
    let r = commands::cmd_learning_batch(&mut c, &sid, None, 7).unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 0);
    assert_eq!(r["already_graduated_song_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["already_graduated_song_ids"][0], sid);
}

#[test]
fn test_learning_batch_relearn_graduated() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 19, now, now, now, 1); // graduated
    let r = commands::cmd_learning_batch(&mut c, &sid, Some(&sid), 7).unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["already_graduated_song_ids"].as_array().unwrap().len(), 0);
    // Verify new record starts at level 7
    let lid = r["created_ids"][0].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "learning", lid, "level,graduated,last_level_up_at").unwrap();
    assert_eq!(g["results"][0]["level"], 7);
    assert_eq!(g["results"][0]["graduated"], 0);
    assert!(g["results"][0]["last_level_up_at"].as_i64().unwrap() > 0);
}

#[test]
fn test_learning_batch_relearn_custom_start_level() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 19, now, now, now, 1);
    let r = commands::cmd_learning_batch(&mut c, &sid, Some(&sid), 5).unwrap();
    let lid = r["created_ids"][0].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "learning", lid, "level").unwrap();
    assert_eq!(g["results"][0]["level"], 5);
}

#[test]
fn test_learning_batch_mixed_new_active_graduated() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s_new = insert_song(&mut c, "New", &aid);
    let s_active = insert_song(&mut c, "Active", &aid);
    let s_grad = insert_song(&mut c, "Grad", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &s_active, 5, now, now, now, 0);
    insert_learning_raw(&mut c, &s_grad, 19, now, now, now, 1);
    let r = commands::cmd_learning_batch(&mut c, &format!("{s_new},{s_active},{s_grad}"), None, 7)
        .unwrap();
    assert_eq!(r["created_ids"].as_array().unwrap().len(), 1); // only s_new
    assert_eq!(r["skipped_song_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["skipped_song_ids"][0], s_active);
    assert_eq!(r["already_graduated_song_ids"].as_array().unwrap().len(), 1);
    assert_eq!(r["already_graduated_song_ids"][0], s_grad);
}

#[test]
fn test_learning_batch_song_not_found() {
    let mut c = test_conn();
    let r = commands::cmd_learning_batch(&mut c, "nonexistent-song-id", None, 7);
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("song not found"));
}

#[test]
fn test_learning_batch_empty_song_ids() {
    let mut c = test_conn();
    let r = commands::cmd_learning_batch(&mut c, "", None, 7);
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("song_ids cannot be empty")
    );
}

// === LEARNING-SONG-REVIEW ===

#[test]
fn test_learning_song_review_generates_html() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "TestArtist");
    let sid = insert_song(&mut c, "TestSong", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);

    // Add a show and link it
    let show_id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    c.execute(
        "INSERT INTO show (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![show_id, "TestShow", now, now],
    )
    .unwrap();
    c.execute(
        "INSERT INTO rel_show_song (show_id, song_id, media_url, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![show_id, sid, "https://example.com/media1", now],
    )
    .unwrap();

    let output_path = std::env::temp_dir().join("test_review.html");
    let output_str = output_path.to_string_lossy().to_string();
    let r = commands::cmd_learning_song_review(&mut c, &output_str, 500, 0).unwrap();

    assert_eq!(r["count"], 1);
    assert_eq!(r["file"], output_str);

    // Verify file was created and contains expected content
    let html = std::fs::read_to_string(&output_path).unwrap();
    assert!(html.contains("TestSong"));
    assert!(html.contains("TestArtist"));
    assert!(html.contains("TestShow"));
    assert!(html.contains("https://example.com/media1"));
    assert!(html.contains("Total due: 1 songs"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_learning_song_review_empty_due() {
    let mut c = test_conn();
    let output_path = std::env::temp_dir().join("test_review_empty.html");
    let output_str = output_path.to_string_lossy().to_string();
    let r = commands::cmd_learning_song_review(&mut c, &output_str, 500, 0).unwrap();

    assert_eq!(r["count"], 0);
    let html = std::fs::read_to_string(&output_path).unwrap();
    assert!(html.contains("Total due: 0 songs"));

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_learning_song_review_deduplicates_media_urls() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);

    let now = jankenoboe::models::now_unix();
    let show_id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO show (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![show_id, "Show1", now, now],
    )
    .unwrap();
    // Same URL in rel_show_song and play_history
    c.execute(
        "INSERT INTO rel_show_song (show_id, song_id, media_url, created_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![show_id, sid, "https://example.com/same", now],
    )
    .unwrap();
    let ph_id = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO play_history (id, show_id, song_id, media_url, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![ph_id, show_id, sid, "https://example.com/same", now],
    )
    .unwrap();

    let output_path = std::env::temp_dir().join("test_review_dedup.html");
    let output_str = output_path.to_string_lossy().to_string();
    let r = commands::cmd_learning_song_review(&mut c, &output_str, 500, 0).unwrap();
    assert_eq!(r["count"], 1);

    let html = std::fs::read_to_string(&output_path).unwrap();
    // The URL should appear only once in the mediaUrls JSON array, not twice
    // Media rendering is now done client-side, so we check the JSON data
    let url_count = html.matches("https://example.com/same").count();
    assert_eq!(url_count, 1);

    std::fs::remove_file(&output_path).ok();
}

// === LEARNING-SONG-LEVELUP-IDS ===

#[test]
fn test_learning_song_levelup_ids_basic() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    let lid = insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);

    let r = commands::cmd_learning_song_levelup_ids(&mut c, &lid).unwrap();
    assert_eq!(r["leveled_up_count"], 1);
    assert_eq!(r["graduated_count"], 0);
    assert_eq!(r["total_processed"], 1);

    // Verify level was incremented
    let g = commands::cmd_get(&mut c, "learning", &lid, "level,last_level_up_at").unwrap();
    assert_eq!(g["results"][0]["level"], 1);
    assert!(g["results"][0]["last_level_up_at"].as_i64().unwrap() > 0);
}

#[test]
fn test_learning_song_levelup_ids_multiple() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    let lid1 = insert_learning_raw(&mut c, &s1, 3, past, past, past, 0);
    let lid2 = insert_learning_raw(&mut c, &s2, 5, past, past, past, 0);

    let ids = format!("{lid1},{lid2}");
    let r = commands::cmd_learning_song_levelup_ids(&mut c, &ids).unwrap();
    assert_eq!(r["leveled_up_count"], 2);
    assert_eq!(r["graduated_count"], 0);
    assert_eq!(r["total_processed"], 2);

    let g1 = commands::cmd_get(&mut c, "learning", &lid1, "level").unwrap();
    assert_eq!(g1["results"][0]["level"], 4);

    let g2 = commands::cmd_get(&mut c, "learning", &lid2, "level").unwrap();
    assert_eq!(g2["results"][0]["level"], 6);
}

#[test]
fn test_learning_song_levelup_ids_graduates_max_level() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    let lid = insert_learning_raw(&mut c, &sid, 19, past, past, past, 0);

    let r = commands::cmd_learning_song_levelup_ids(&mut c, &lid).unwrap();
    assert_eq!(r["leveled_up_count"], 0);
    assert_eq!(r["graduated_count"], 1);
    assert_eq!(r["total_processed"], 1);

    let g = commands::cmd_get(&mut c, "learning", &lid, "graduated").unwrap();
    assert_eq!(g["results"][0]["graduated"], 1);
}

#[test]
fn test_learning_song_levelup_ids_mixed_levelup_and_graduate() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    let lid1 = insert_learning_raw(&mut c, &s1, 0, past, past, 0, 0);
    let lid2 = insert_learning_raw(&mut c, &s2, 19, past, past, past, 0);

    let ids = format!("{lid1},{lid2}");
    let r = commands::cmd_learning_song_levelup_ids(&mut c, &ids).unwrap();
    assert_eq!(r["leveled_up_count"], 1);
    assert_eq!(r["graduated_count"], 1);
    assert_eq!(r["total_processed"], 2);

    let g1 = commands::cmd_get(&mut c, "learning", &lid1, "level").unwrap();
    assert_eq!(g1["results"][0]["level"], 1);

    let g2 = commands::cmd_get(&mut c, "learning", &lid2, "graduated").unwrap();
    assert_eq!(g2["results"][0]["graduated"], 1);
}

#[test]
fn test_learning_song_levelup_ids_not_found() {
    let mut c = test_conn();
    let r = commands::cmd_learning_song_levelup_ids(&mut c, "nonexistent-id");
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_learning_song_levelup_ids_empty() {
    let mut c = test_conn();
    let r = commands::cmd_learning_song_levelup_ids(&mut c, "");
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("ids cannot be empty"));
}

#[test]
fn test_learning_song_levelup_ids_already_graduated() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    let lid = insert_learning_raw(&mut c, &sid, 19, now, now, now, 1); // already graduated

    let r = commands::cmd_learning_song_levelup_ids(&mut c, &lid);
    assert!(r.is_err());
    assert!(r.unwrap_err().to_string().contains("already graduated"));
}

#[test]
fn test_learning_song_levelup_ids_does_not_require_due() {
    // Unlike levelup-due, levelup-ids should work regardless of due status
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    // Level 5, recently updated (NOT due)
    let lid = insert_learning_raw(&mut c, &sid, 5, now, now, now, 0);

    let r = commands::cmd_learning_song_levelup_ids(&mut c, &lid).unwrap();
    assert_eq!(r["leveled_up_count"], 1);
    assert_eq!(r["total_processed"], 1);

    let g = commands::cmd_get(&mut c, "learning", &lid, "level").unwrap();
    assert_eq!(g["results"][0]["level"], 6);
}

// === LEARNING-SONG-REVIEW returns learning_ids ===

#[test]
fn test_learning_song_review_returns_learning_ids() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let past = jankenoboe::models::now_unix() - 400;
    let lid = insert_learning_raw(&mut c, &sid, 0, past, past, 0, 0);

    let output_path = std::env::temp_dir().join("test_review_ids.html");
    let output_str = output_path.to_string_lossy().to_string();
    let r = commands::cmd_learning_song_review(&mut c, &output_str, 500, 0).unwrap();

    assert_eq!(r["count"], 1);
    let learning_ids = r["learning_ids"].as_array().unwrap();
    assert_eq!(learning_ids.len(), 1);
    assert_eq!(learning_ids[0], lid);

    std::fs::remove_file(&output_path).ok();
}

#[test]
fn test_learning_song_review_empty_learning_ids() {
    let mut c = test_conn();
    let output_path = std::env::temp_dir().join("test_review_empty_ids.html");
    let output_str = output_path.to_string_lossy().to_string();
    let r = commands::cmd_learning_song_review(&mut c, &output_str, 500, 0).unwrap();

    assert_eq!(r["count"], 0);
    let learning_ids = r["learning_ids"].as_array().unwrap();
    assert_eq!(learning_ids.len(), 0);

    std::fs::remove_file(&output_path).ok();
}

// === LEARNING-BY-SONG-IDS ===

#[test]
fn test_learning_by_song_ids_single() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    let lid = insert_learning_raw(&mut c, &sid, 5, now, now, now, 0);

    let r = commands::cmd_learning_by_song_ids(&mut c, &sid).unwrap();
    assert_eq!(r["count"], 1);
    assert_eq!(r["results"][0]["id"], lid);
    assert_eq!(r["results"][0]["song_id"], sid);
    assert_eq!(r["results"][0]["song_name"], "S");
    assert_eq!(r["results"][0]["level"], 5);
    assert_eq!(r["results"][0]["graduated"], 0);
    assert_eq!(r["results"][0]["wait_days"], 1);
}

#[test]
fn test_learning_by_song_ids_multiple_songs() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let s1 = insert_song(&mut c, "S1", &aid);
    let s2 = insert_song(&mut c, "S2", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &s1, 3, now, now, now, 0);
    insert_learning_raw(&mut c, &s2, 10, now, now, now, 0);

    let r = commands::cmd_learning_by_song_ids(&mut c, &format!("{s1},{s2}")).unwrap();
    assert_eq!(r["count"], 2);
    // Ordered by level DESC
    assert_eq!(r["results"][0]["level"], 10);
    assert_eq!(r["results"][1]["level"], 3);
}

#[test]
fn test_learning_by_song_ids_includes_graduated() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    insert_learning_raw(&mut c, &sid, 19, now, now, now, 1);

    let r = commands::cmd_learning_by_song_ids(&mut c, &sid).unwrap();
    assert_eq!(r["count"], 1);
    assert_eq!(r["results"][0]["graduated"], 1);
}

#[test]
fn test_learning_by_song_ids_multiple_records_per_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let now = jankenoboe::models::now_unix();
    // Graduated record + active re-learn record
    insert_learning_raw(&mut c, &sid, 19, now, now, now, 1);
    insert_learning_raw(&mut c, &sid, 7, now, now, now, 0);

    let r = commands::cmd_learning_by_song_ids(&mut c, &sid).unwrap();
    assert_eq!(r["count"], 2);
}

#[test]
fn test_learning_by_song_ids_no_learning_records() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);

    let r = commands::cmd_learning_by_song_ids(&mut c, &sid).unwrap();
    assert_eq!(r["count"], 0);
    assert_eq!(r["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_learning_by_song_ids_empty() {
    let mut c = test_conn();
    let r = commands::cmd_learning_by_song_ids(&mut c, "");
    assert!(r.is_err());
    assert!(
        r.unwrap_err()
            .to_string()
            .contains("song_ids cannot be empty")
    );
}

#[test]
fn test_learning_by_song_ids_nonexistent_song() {
    let mut c = test_conn();
    // Song doesn't exist in song table, but no learning records either — returns empty
    let r = commands::cmd_learning_by_song_ids(&mut c, "nonexistent-id").unwrap();
    assert_eq!(r["count"], 0);
}

// === SQL INJECTION PREVENTION ===

#[test]
fn test_sql_injection_table_name() {
    let mut c = test_conn();
    assert!(commands::cmd_get(&mut c, "artist; DROP TABLE artist;--", "x", "id").is_err());
}

#[test]
fn test_sql_injection_field_name() {
    let mut c = test_conn();
    assert!(commands::cmd_get(&mut c, "artist", "x", "id; DROP TABLE artist").is_err());
}

#[test]
fn test_sql_injection_search_term_key() {
    let mut c = test_conn();
    let err = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name; DROP TABLE artist":{"value":"x"}}"#,
        "id",
    )
    .unwrap_err()
    .to_string();
    assert_eq!(
        err,
        "Invalid term key for artist: name; DROP TABLE artist. Allowed: name, name_context"
    );
}

#[test]
fn test_sql_injection_in_search_value() {
    let mut c = test_conn();
    // This should not cause an error - the value is parameterized
    insert_artist(&mut c, "test");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"'; DROP TABLE artist; --"}}"#,
        "id,name",
    )
    .unwrap();
    // Should return empty results, not crash
    assert_eq!(r["results"].as_array().unwrap().len(), 0);
}
