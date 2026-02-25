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

fn insert_show(conn: &mut Connection, name: &str, vintage: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    conn.execute(
        "INSERT INTO show (id, name, vintage, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, vintage, now, now],
    )
    .unwrap();
    id
}

fn insert_learning(
    conn: &mut Connection,
    song_id: &str,
    level: i64,
    last_up: i64,
    grad: i64,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    let path = "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]";
    conn.execute(
        "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, song_id, level, now, now, last_up, path, grad],
    ).unwrap();
    id
}

// === CREATE ===

#[test]
fn test_create_artist() {
    let mut c = test_conn();
    let r = commands::cmd_create(&mut c, "artist", r#"{"name":"ChoQMay"}"#).unwrap();
    let id = r["id"].as_str().unwrap();
    assert!(!id.is_empty());
    let g = commands::cmd_get(&mut c, "artist", id, "name,status,created_at,updated_at").unwrap();
    let rec = &g["results"][0];
    assert_eq!(rec["name"], "ChoQMay");
    assert_eq!(rec["status"], 0);
    assert!(rec["created_at"].as_i64().unwrap() > 0);
    assert!(rec["updated_at"].as_i64().unwrap() > 0);
}

#[test]
fn test_create_show_all_fields() {
    let mut c = test_conn();
    let r = commands::cmd_create(
        &mut c,
        "show",
        r#"{"name":"Sign","name_romaji":"Yubi","vintage":"Winter 2024","s_type":"TV"}"#,
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "show", id, "name,vintage,s_type").unwrap();
    assert_eq!(g["results"][0]["name"], "Sign");
    assert_eq!(g["results"][0]["vintage"], "Winter 2024");
    assert_eq!(g["results"][0]["s_type"], "TV");
}

#[test]
fn test_create_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let r = commands::cmd_create(
        &mut c,
        "song",
        &format!(r#"{{"name":"snowspring","artist_id":"{aid}"}}"#),
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "song", id, "name,artist_id").unwrap();
    assert_eq!(g["results"][0]["name"], "snowspring");
    assert_eq!(g["results"][0]["artist_id"], aid);
}

#[test]
fn test_create_play_history() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    let r = commands::cmd_create(
        &mut c,
        "play_history",
        &format!(r#"{{"show_id":"{shid}","song_id":"{sid}","media_url":"https://ex.com"}}"#),
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "play_history", id, "show_id,song_id,media_url").unwrap();
    assert_eq!(g["results"][0]["show_id"], shid);
    assert_eq!(g["results"][0]["media_url"], "https://ex.com");
}

#[test]
fn test_create_learning_defaults() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let r = commands::cmd_create(&mut c, "learning", &format!(r#"{{"song_id":"{sid}"}}"#)).unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(
        &mut c,
        "learning",
        id,
        "song_id,level,graduated,last_level_up_at,level_up_path",
    )
    .unwrap();
    let rec = &g["results"][0];
    assert_eq!(rec["song_id"], sid);
    assert_eq!(rec["level"], 0);
    assert_eq!(rec["graduated"], 0);
    assert_eq!(rec["last_level_up_at"], 0);
    assert_eq!(
        rec["level_up_path"],
        "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]"
    );
}

#[test]
fn test_create_rel_show_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    let r = commands::cmd_create(
        &mut c,
        "rel_show_song",
        &format!(r#"{{"show_id":"{shid}","song_id":"{sid}","media_url":"https://ex.com"}}"#),
    )
    .unwrap();
    assert_eq!(r["id"].as_str().unwrap(), format!("{shid}:{sid}"));
}

#[test]
fn test_create_invalid_table() {
    let mut c = test_conn();
    assert!(commands::cmd_create(&mut c, "bad", r#"{"name":"t"}"#).is_err());
}

#[test]
fn test_create_invalid_field() {
    let mut c = test_conn();
    assert!(
        commands::cmd_create(&mut c, "artist", r#"{"name":"t","password":"s"}"#)
            .unwrap_err()
            .to_string()
            .contains("Invalid field")
    );
}

// === UPDATE ===

#[test]
fn test_update_artist_name() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "Old");
    let r = commands::cmd_update(&mut c, "artist", &id, r#"{"name":"New"}"#).unwrap();
    assert_eq!(r["updated"], true);
    let g = commands::cmd_get(&mut c, "artist", &id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "New");
}

// === JSON VALUE TYPE EDGE CASES ===

#[test]
fn test_update_with_boolean_value() {
    // Tests json_value_to_sql Bool branch: {"graduated": true} → 1
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 19, 1000, 0);
    // Use a JSON boolean `true` instead of number `1`
    commands::cmd_update(&mut c, "learning", &lid, r#"{"graduated":true}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "graduated").unwrap();
    assert_eq!(g["results"][0]["graduated"], 1);
}

#[test]
fn test_create_with_null_value() {
    // Tests json_value_to_sql Null branch
    let mut c = test_conn();
    let r = commands::cmd_create(&mut c, "artist", r#"{"name":"X","name_context":null}"#).unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "artist", id, "name_context").unwrap();
    // null value in SQLite — returned as empty or null
    let val = &g["results"][0]["name_context"];
    assert!(val.is_null() || val.as_str() == Some(""));
}

#[test]
fn test_search_with_float_column() {
    // Tests row_value_at float path by inserting a float value directly
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // Insert a learning record with a REAL (float) value in the level column
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    c.execute(
        "INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![id, sid, 2.5f64, now, now, 0, "[1,2,3]", 0],
    ).unwrap();
    let r = commands::cmd_get(&mut c, "learning", &id, "level").unwrap();
    // Float 2.5 should be returned
    let level = &r["results"][0]["level"];
    assert_eq!(level.as_f64().unwrap(), 2.5);
}

#[test]
fn test_get_with_null_column_value() {
    // Tests row_value_at null path by having a NULL column
    let mut c = test_conn();
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    // Insert artist with NULL name_context
    c.execute(
        "INSERT INTO artist (id, name, name_context, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, ?4)",
        rusqlite::params![id, "Test", now, now],
    ).unwrap();
    let r = commands::cmd_get(&mut c, "artist", &id, "name_context").unwrap();
    assert!(r["results"][0]["name_context"].is_null());
}

#[test]
fn test_create_with_json_array_value() {
    // Tests json_value_to_sql catch-all branch (array → to_string)
    // level_up_path accepts string, but passing a JSON array exercises the fallback
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let r = commands::cmd_create(
        &mut c,
        "learning",
        &format!(r#"{{"song_id":"{sid}","level_up_path":[1,2,3]}}"#),
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "learning", id, "level_up_path").unwrap();
    assert_eq!(g["results"][0]["level_up_path"], "[1,2,3]");
}

#[test]
fn test_create_with_float_number() {
    // Tests json_value_to_sql f64 branch
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 0, 0, 0);
    commands::cmd_update(&mut c, "learning", &lid, r#"{"level":3.5}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "level").unwrap();
    // SQLite will store 3.5 as REAL
    assert_eq!(g["results"][0]["level"].as_f64().unwrap(), 3.5);
}

#[test]
fn test_update_learning_level_sets_last_level_up_at() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 3, 0, 0);
    commands::cmd_update(&mut c, "learning", &lid, r#"{"level":8}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "level,last_level_up_at").unwrap();
    assert_eq!(g["results"][0]["level"], 8);
    assert!(g["results"][0]["last_level_up_at"].as_i64().unwrap() > 0);
}

#[test]
fn test_update_learning_graduate() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 19, 1000, 0);
    commands::cmd_update(&mut c, "learning", &lid, r#"{"graduated":1}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "graduated").unwrap();
    assert_eq!(g["results"][0]["graduated"], 1);
}

#[test]
fn test_update_graduated_does_not_set_last_level_up_at() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 19, 0, 0);
    commands::cmd_update(&mut c, "learning", &lid, r#"{"graduated":1}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "last_level_up_at").unwrap();
    assert_eq!(g["results"][0]["last_level_up_at"], 0);
}

#[test]
fn test_update_not_found() {
    let mut c = test_conn();
    assert!(
        commands::cmd_update(&mut c, "artist", "no", r#"{"name":"X"}"#)
            .unwrap_err()
            .to_string()
            .contains("Record not found")
    );
}

#[test]
fn test_update_invalid_table() {
    let mut c = test_conn();
    assert!(commands::cmd_update(&mut c, "rel_show_song", "x", r#"{"show_id":"y"}"#).is_err());
}

#[test]
fn test_update_invalid_field() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "A");
    assert!(
        commands::cmd_update(&mut c, "artist", &id, r#"{"id":"new"}"#)
            .unwrap_err()
            .to_string()
            .contains("Invalid field")
    );
}

#[test]
fn test_update_empty_data() {
    let mut c = test_conn();
    assert!(
        commands::cmd_update(&mut c, "artist", "x", r#"{}"#)
            .unwrap_err()
            .to_string()
            .contains("data cannot be empty")
    );
}

// === DELETE ===

#[test]
fn test_delete_artist() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "A");
    let r = commands::cmd_delete(&mut c, "artist", &id).unwrap();
    assert_eq!(r["deleted"], true);
    let g = commands::cmd_get(&mut c, "artist", &id, "id").unwrap();
    assert_eq!(g["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_delete_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    assert_eq!(
        commands::cmd_delete(&mut c, "song", &sid).unwrap()["deleted"],
        true
    );
}

#[test]
fn test_delete_not_found() {
    let mut c = test_conn();
    assert!(
        commands::cmd_delete(&mut c, "artist", "no")
            .unwrap_err()
            .to_string()
            .contains("Record not found")
    );
}

#[test]
fn test_delete_invalid_table() {
    let mut c = test_conn();
    assert!(commands::cmd_delete(&mut c, "show", "x").is_err());
    assert!(commands::cmd_delete(&mut c, "learning", "x").is_err());
    assert!(commands::cmd_delete(&mut c, "play_history", "x").is_err());
}

// === BULK-REASSIGN ===

#[test]
fn test_bulk_reassign_by_song_ids() {
    let mut c = test_conn();
    let a1 = insert_artist(&mut c, "A1");
    let a2 = insert_artist(&mut c, "A2");
    let s1 = insert_song(&mut c, "S1", &a1);
    let s2 = insert_song(&mut c, "S2", &a1);
    let r = commands::cmd_bulk_reassign(&mut c, Some(&format!("{s1},{s2}")), Some(&a2), None, None)
        .unwrap();
    assert_eq!(r["reassigned_count"], 2);
    let g1 = commands::cmd_get(&mut c, "song", &s1, "artist_id").unwrap();
    assert_eq!(g1["results"][0]["artist_id"], a2);
    let g2 = commands::cmd_get(&mut c, "song", &s2, "artist_id").unwrap();
    assert_eq!(g2["results"][0]["artist_id"], a2);
}

#[test]
fn test_bulk_reassign_by_artist() {
    let mut c = test_conn();
    let a1 = insert_artist(&mut c, "A1");
    let a2 = insert_artist(&mut c, "A2");
    insert_song(&mut c, "S1", &a1);
    insert_song(&mut c, "S2", &a1);
    insert_song(&mut c, "S3", &a2);
    let r = commands::cmd_bulk_reassign(&mut c, None, None, Some(&a1), Some(&a2)).unwrap();
    assert_eq!(r["reassigned_count"], 2);
}

#[test]
fn test_bulk_reassign_invalid_args() {
    let mut c = test_conn();
    // No valid combination
    assert!(commands::cmd_bulk_reassign(&mut c, None, None, None, None).is_err());
    // Mixed modes
    assert!(commands::cmd_bulk_reassign(&mut c, Some("x"), Some("y"), Some("z"), None).is_err());
}

#[test]
fn test_bulk_reassign_empty_song_ids() {
    let mut c = test_conn();
    assert!(
        commands::cmd_bulk_reassign(&mut c, Some(""), Some("any-id"), None, None)
            .unwrap_err()
            .to_string()
            .contains("song_ids cannot be empty")
    );
}

// === UPDATE ADDITIONAL TABLE BRANCHES ===

#[test]
fn test_update_show_name() {
    let mut c = test_conn();
    let id = insert_show(&mut c, "Old Show", "2024");
    let r = commands::cmd_update(&mut c, "show", &id, r#"{"name":"New Show"}"#).unwrap();
    assert_eq!(r["updated"], true);
    let g = commands::cmd_get(&mut c, "show", &id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "New Show");
}

#[test]
fn test_update_song_name() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "Old Song", &aid);
    let r = commands::cmd_update(&mut c, "song", &sid, r#"{"name":"New Song"}"#).unwrap();
    assert_eq!(r["updated"], true);
    let g = commands::cmd_get(&mut c, "song", &sid, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "New Song");
}

#[test]
fn test_update_play_history_media_url() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    let r = commands::cmd_create(
        &mut c,
        "play_history",
        &format!(r#"{{"show_id":"{shid}","song_id":"{sid}","media_url":"https://old.com"}}"#),
    )
    .unwrap();
    let ph_id = r["id"].as_str().unwrap();
    let r2 = commands::cmd_update(
        &mut c,
        "play_history",
        ph_id,
        r#"{"media_url":"https://new.com"}"#,
    )
    .unwrap();
    assert_eq!(r2["updated"], true);
    let g = commands::cmd_get(&mut c, "play_history", ph_id, "media_url").unwrap();
    assert_eq!(g["results"][0]["media_url"], "https://new.com");
}

// === CREATE/UPDATE WITH NUMERIC VALUES ===

#[test]
fn test_create_with_integer_value() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    // level_up_path is a text field, but level in learning creation uses integer
    let r = commands::cmd_create(
        &mut c,
        "learning",
        &format!(r#"{{"song_id":"{sid}","level_up_path":"[1,2,3]"}}"#),
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "learning", id, "level_up_path").unwrap();
    assert_eq!(g["results"][0]["level_up_path"], "[1,2,3]");
}

#[test]
fn test_create_invalid_json_data() {
    let mut c = test_conn();
    assert!(commands::cmd_create(&mut c, "artist", "not json").is_err());
}

#[test]
fn test_update_invalid_json_data() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "A");
    assert!(commands::cmd_update(&mut c, "artist", &id, "not json").is_err());
}

// === CREATE rel_show_song EDGE CASES ===

#[test]
fn test_create_rel_show_song_without_media_url() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    let r = commands::cmd_create(
        &mut c,
        "rel_show_song",
        &format!(r#"{{"show_id":"{shid}","song_id":"{sid}"}}"#),
    )
    .unwrap();
    assert_eq!(r["id"].as_str().unwrap(), format!("{shid}:{sid}"));
}

// === MODELS: INVALID TABLE FOR CREATE/UPDATE ===

#[test]
fn test_create_invalid_table_for_create_data_fields() {
    let mut c = test_conn();
    assert!(
        commands::cmd_create(&mut c, "bad_table", r#"{"name":"x"}"#)
            .unwrap_err()
            .to_string()
            .contains("Invalid table")
    );
}

#[test]
fn test_update_invalid_table_for_update_data_fields() {
    let mut c = test_conn();
    assert!(
        commands::cmd_update(&mut c, "bad_table", "x", r#"{"name":"x"}"#)
            .unwrap_err()
            .to_string()
            .contains("Invalid table")
    );
}

// === URL PERCENT-ENCODING IN --data ===

#[test]
fn test_create_artist_url_encoded_single_quote() {
    let mut c = test_conn();
    // Name with single quote: "Ado's Music" encoded as "Ado%27s%20Music"
    let r = commands::cmd_create(&mut c, "artist", r#"{"name":"Ado%27s%20Music"}"#).unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "artist", id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "Ado's Music");
}

#[test]
fn test_create_artist_url_encoded_double_quote() {
    let mut c = test_conn();
    // Name with double quote: The "Best" encoded as The%20%22Best%22
    let r = commands::cmd_create(&mut c, "artist", r#"{"name":"The%20%22Best%22"}"#).unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "artist", id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "The \"Best\"");
}

#[test]
fn test_create_song_url_encoded_parentheses() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    // "Fuwa Fuwa Time (5-nin Ver.)" encoded
    let r = commands::cmd_create(
        &mut c,
        "song",
        &format!(r#"{{"name":"Fuwa%20Fuwa%20Time%20%285-nin%20Ver.%29","artist_id":"{aid}"}}"#),
    )
    .unwrap();
    let id = r["id"].as_str().unwrap();
    let g = commands::cmd_get(&mut c, "song", id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "Fuwa Fuwa Time (5-nin Ver.)");
}

#[test]
fn test_update_artist_url_encoded_single_quote() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "Old");
    // Update with single quote: "it%27s%20new"
    let r = commands::cmd_update(&mut c, "artist", &id, r#"{"name":"it%27s%20new"}"#).unwrap();
    assert_eq!(r["updated"], true);
    let g = commands::cmd_get(&mut c, "artist", &id, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "it's new");
}

#[test]
fn test_update_song_url_encoded_special_chars() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "Old", &aid);
    // Update with exclamation and ampersand: "Rock%20%26%20Roll%21"
    let r =
        commands::cmd_update(&mut c, "song", &sid, r#"{"name":"Rock%20%26%20Roll%21"}"#).unwrap();
    assert_eq!(r["updated"], true);
    let g = commands::cmd_get(&mut c, "song", &sid, "name").unwrap();
    assert_eq!(g["results"][0]["name"], "Rock & Roll!");
}

#[test]
fn test_create_data_integer_not_decoded() {
    // Ensure integer values are NOT affected by URL decoding (only strings are decoded)
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 0, 0, 0);
    commands::cmd_update(&mut c, "learning", &lid, r#"{"level":5}"#).unwrap();
    let g = commands::cmd_get(&mut c, "learning", &lid, "level").unwrap();
    assert_eq!(g["results"][0]["level"], 5);
}

#[test]
fn test_create_invalid_url_encoding() {
    let mut c = test_conn();
    // Invalid percent-encoding: "%ZZ" is not valid hex
    let err = commands::cmd_create(&mut c, "artist", r#"{"name":"Bad%ZZvalue"}"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("URL decoding error"));
}

#[test]
fn test_update_invalid_url_encoding() {
    let mut c = test_conn();
    let id = insert_artist(&mut c, "A");
    let err = commands::cmd_update(&mut c, "artist", &id, r#"{"name":"Bad%ZZvalue"}"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("URL decoding error"));
}

#[test]
fn test_create_and_search_roundtrip_with_quotes() {
    let mut c = test_conn();
    // Create artist with URL-encoded name containing single quote
    let r = commands::cmd_create(&mut c, "artist", r#"{"name":"Can%27t%20Stop"}"#).unwrap();
    let id = r["id"].as_str().unwrap();
    // Search using URL-encoded value
    let s = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"Can%27t%20Stop"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(s["results"].as_array().unwrap().len(), 1);
    assert_eq!(s["results"][0]["id"], id);
    assert_eq!(s["results"][0]["name"], "Can't Stop");
}
