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

fn insert_play_history(
    conn: &mut Connection,
    show_id: &str,
    song_id: &str,
    media_url: &str,
) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = jankenoboe::models::now_unix();
    conn.execute(
        "INSERT INTO play_history (id, show_id, song_id, created_at, media_url) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, show_id, song_id, now, media_url],
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

// === GET ===

#[test]
fn test_get_artist_by_id() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "ChoQMay");
    let r = commands::cmd_get(&mut c, "artist", &aid, "id,name").unwrap();
    let results = r["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], aid);
    assert_eq!(results[0]["name"], "ChoQMay");
}

#[test]
fn test_get_song_multiple_fields() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let r = commands::cmd_get(&mut c, "song", &sid, "id,name,artist_id").unwrap();
    assert_eq!(r["results"][0]["name"], "S");
    assert_eq!(r["results"][0]["artist_id"], aid);
}

#[test]
fn test_get_nonexistent_returns_empty() {
    let mut c = test_conn();
    let r = commands::cmd_get(&mut c, "artist", "no-such-id", "id,name").unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_get_invalid_table() {
    let mut c = test_conn();
    assert!(
        commands::cmd_get(&mut c, "bad", "x", "id")
            .unwrap_err()
            .to_string()
            .contains("Invalid table")
    );
}

#[test]
fn test_get_invalid_field() {
    let mut c = test_conn();
    assert!(
        commands::cmd_get(&mut c, "artist", "x", "id,password")
            .unwrap_err()
            .to_string()
            .contains("Invalid field")
    );
}

#[test]
fn test_get_empty_fields() {
    let mut c = test_conn();
    assert!(
        commands::cmd_get(&mut c, "artist", "x", "")
            .unwrap_err()
            .to_string()
            .contains("fields cannot be empty")
    );
}

#[test]
fn test_get_learning_record() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let lid = insert_learning(&mut c, &sid, 5, 0, 0);
    let r = commands::cmd_get(&mut c, "learning", &lid, "level,graduated").unwrap();
    assert_eq!(r["results"][0]["level"], 5);
    assert_eq!(r["results"][0]["graduated"], 0);
}

// === SEARCH ===

#[test]
fn test_search_exact_match() {
    let mut c = test_conn();
    insert_artist(&mut c, "ChoQMay");
    insert_artist(&mut c, "Minami");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"ChoQMay"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "ChoQMay");
}

#[test]
fn test_search_exact_is_case_sensitive() {
    let mut c = test_conn();
    insert_artist(&mut c, "Minami");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"minami","match":"exact"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_search_exact_i() {
    let mut c = test_conn();
    insert_artist(&mut c, "Minami");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"minami","match":"exact-i"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "Minami");
}

#[test]
fn test_search_starts_with() {
    let mut c = test_conn();
    insert_artist(&mut c, "Minami");
    insert_artist(&mut c, "Misa");
    insert_artist(&mut c, "ZZZ");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"min","match":"starts-with"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "Minami");
}

#[test]
fn test_search_ends_with() {
    let mut c = test_conn();
    insert_artist(&mut c, "Minami");
    insert_artist(&mut c, "Konami");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"ami","match":"ends-with"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 2);
}

#[test]
fn test_search_contains() {
    let mut c = test_conn();
    insert_artist(&mut c, "Minami");
    insert_artist(&mut c, "Other");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"nam","match":"contains"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
}

#[test]
fn test_search_multiple_and_conditions() {
    let mut c = test_conn();
    insert_show(&mut c, "A Sign of Affection", "Winter 2024");
    insert_show(&mut c, "Sign Something", "Spring 2024");
    insert_show(&mut c, "A Sign of Affection", "Summer 2020");
    let r = commands::cmd_search(
        &mut c, "show",
        r#"{"name":{"value":"sign","match":"contains"},"vintage":{"value":"2024","match":"ends-with"}}"#,
        "id,name,vintage",
    ).unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 2);
}

#[test]
fn test_search_rel_show_song() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let sid = insert_song(&mut c, "S", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    let now = jankenoboe::models::now_unix();
    c.execute(
        "INSERT INTO rel_show_song (show_id, song_id, created_at) VALUES (?1, ?2, ?3)",
        rusqlite::params![shid, sid, now],
    )
    .unwrap();
    let r = commands::cmd_search(
        &mut c,
        "rel_show_song",
        &format!(r#"{{"show_id":{{"value":"{shid}"}},"song_id":{{"value":"{sid}"}}}}"#),
        "show_id,song_id",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
}

#[test]
fn test_search_song_by_artist_id() {
    let mut c = test_conn();
    let a1 = insert_artist(&mut c, "A1");
    let a2 = insert_artist(&mut c, "A2");
    insert_song(&mut c, "S1", &a1);
    insert_song(&mut c, "S2", &a1);
    insert_song(&mut c, "S3", &a2);
    let r = commands::cmd_search(
        &mut c,
        "song",
        &format!(r#"{{"artist_id":{{"value":"{a1}"}}}}"#),
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 2);
}

#[test]
fn test_search_invalid_table() {
    let mut c = test_conn();
    assert!(commands::cmd_search(&mut c, "learning", r#"{"name":{"value":"t"}}"#, "id").is_err());
}

#[test]
fn test_search_invalid_column() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{"id":{"value":"t"}}"#, "id,name")
            .unwrap_err()
            .to_string()
            .contains("Invalid search column")
    );
}

#[test]
fn test_search_invalid_match_mode() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(
            &mut c,
            "artist",
            r#"{"name":{"value":"t","match":"regex"}}"#,
            "id,name"
        )
        .unwrap_err()
        .to_string()
        .contains("Invalid match mode")
    );
}

#[test]
fn test_search_empty_term() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{}"#, "id,name")
            .unwrap_err()
            .to_string()
            .contains("term cannot be empty")
    );
}

#[test]
fn test_search_term_condition_not_object() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{"name":"string_val"}"#, "id,name")
            .unwrap_err()
            .to_string()
            .contains("must be an object")
    );
}

#[test]
fn test_search_term_condition_missing_value() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{"name":{"match":"exact"}}"#, "id,name")
            .unwrap_err()
            .to_string()
            .contains("must have a 'value' string")
    );
}

#[test]
fn test_search_term_value_not_string() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{"name":{"value":123}}"#, "id,name")
            .unwrap_err()
            .to_string()
            .contains("must have a 'value' string")
    );
}

#[test]
fn test_search_invalid_json() {
    let mut c = test_conn();
    assert!(commands::cmd_search(&mut c, "artist", "not json", "id,name").is_err());
}

#[test]
fn test_search_url_encoded_value() {
    let mut c = test_conn();
    insert_artist(&mut c, "it's a test");
    insert_artist(&mut c, "Other");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"it%27s%20a%20test"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "it's a test");
}

#[test]
fn test_search_url_encoded_value_contains() {
    let mut c = test_conn();
    insert_artist(&mut c, "K-On! (Movie)");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"K-On%21","match":"starts-with"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "K-On! (Movie)");
}

#[test]
fn test_search_plain_value_still_works() {
    let mut c = test_conn();
    insert_artist(&mut c, "ChoQMay");
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"ChoQMay"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "ChoQMay");
}

#[test]
fn test_search_url_encoded_single_quote() {
    let mut c = test_conn();
    insert_artist(&mut c, "Ado's Music");
    insert_artist(&mut c, "Other");
    // Single quote encoded as %27
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"Ado%27s%20Music"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "Ado's Music");
}

#[test]
fn test_search_url_encoded_double_quote() {
    let mut c = test_conn();
    insert_artist(&mut c, "The \"Best\" Artist");
    // Double quote encoded as %22
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"The%20%22Best%22%20Artist"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "The \"Best\" Artist");
}

#[test]
fn test_search_url_encoded_quotes_contains() {
    let mut c = test_conn();
    insert_artist(&mut c, "Fuwa Fuwa Time (5-nin Ver.)");
    // Parentheses and space encoded
    let r = commands::cmd_search(
        &mut c,
        "artist",
        r#"{"name":{"value":"5-nin%20Ver.","match":"contains"}}"#,
        "id,name",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["name"], "Fuwa Fuwa Time (5-nin Ver.)");
}

#[test]
fn test_search_empty_fields() {
    let mut c = test_conn();
    assert!(
        commands::cmd_search(&mut c, "artist", r#"{"name":{"value":"x"}}"#, "")
            .unwrap_err()
            .to_string()
            .contains("fields cannot be empty")
    );
}

// === SEARCH play_history ===

#[test]
fn test_search_play_history_by_song_id() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let song1 = insert_song(&mut c, "S1", &aid);
    let song2 = insert_song(&mut c, "S2", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    insert_play_history(&mut c, &shid, &song1, "https://example.com/1");
    insert_play_history(&mut c, &shid, &song1, "https://example.com/2");
    insert_play_history(&mut c, &shid, &song2, "https://example.com/3");
    let r = commands::cmd_search(
        &mut c,
        "play_history",
        &format!(r#"{{"song_id":{{"value":"{song1}"}}}}"#),
        "id,song_id,media_url",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 2);
}

#[test]
fn test_search_play_history_by_show_id() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let song1 = insert_song(&mut c, "S1", &aid);
    let sh1 = insert_show(&mut c, "Show1", "2024");
    let sh2 = insert_show(&mut c, "Show2", "2024");
    insert_play_history(&mut c, &sh1, &song1, "");
    insert_play_history(&mut c, &sh2, &song1, "");
    let r = commands::cmd_search(
        &mut c,
        "play_history",
        &format!(r#"{{"show_id":{{"value":"{sh1}"}}}}"#),
        "id,show_id,song_id",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["show_id"], sh1);
}

#[test]
fn test_search_play_history_by_both_ids() {
    let mut c = test_conn();
    let aid = insert_artist(&mut c, "A");
    let song1 = insert_song(&mut c, "S1", &aid);
    let song2 = insert_song(&mut c, "S2", &aid);
    let shid = insert_show(&mut c, "Sh", "2024");
    insert_play_history(&mut c, &shid, &song1, "url1");
    insert_play_history(&mut c, &shid, &song2, "url2");
    let r = commands::cmd_search(
        &mut c,
        "play_history",
        &format!(r#"{{"show_id":{{"value":"{shid}"}},"song_id":{{"value":"{song1}"}}}}"#),
        "id,media_url",
    )
    .unwrap();
    assert_eq!(r["results"].as_array().unwrap().len(), 1);
    assert_eq!(r["results"][0]["media_url"], "url1");
}

// === DUPLICATES ===

#[test]
fn test_duplicates_artist() {
    let mut c = test_conn();
    let a1 = insert_artist(&mut c, "Minami");
    let a2 = insert_artist(&mut c, "MINAMI");
    insert_artist(&mut c, "Unique");
    insert_song(&mut c, "S1", &a1);
    insert_song(&mut c, "S2", &a1);
    insert_song(&mut c, "S3", &a2);
    let r = commands::cmd_duplicates(&mut c, "artist").unwrap();
    let dups = r["duplicates"].as_array().unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0]["name"], "minami");
    let recs = dups[0]["records"].as_array().unwrap();
    assert_eq!(recs.len(), 2);
    let mut counts: Vec<i64> = recs
        .iter()
        .map(|r| r["song_count"].as_i64().unwrap())
        .collect();
    counts.sort();
    assert_eq!(counts, vec![1, 2]);
}

#[test]
fn test_duplicates_none() {
    let mut c = test_conn();
    insert_artist(&mut c, "A1");
    insert_artist(&mut c, "A2");
    let r = commands::cmd_duplicates(&mut c, "artist").unwrap();
    assert_eq!(r["duplicates"].as_array().unwrap().len(), 0);
}

#[test]
fn test_duplicates_show() {
    let mut c = test_conn();
    insert_show(&mut c, "K-On!", "Spring 2009");
    insert_show(&mut c, "k-on!", "Fall 2009");
    let r = commands::cmd_duplicates(&mut c, "show").unwrap();
    assert_eq!(r["duplicates"].as_array().unwrap().len(), 1);
}

#[test]
fn test_duplicates_invalid_table() {
    let mut c = test_conn();
    assert!(commands::cmd_duplicates(&mut c, "learning").is_err());
}

#[test]
fn test_duplicates_song() {
    let mut c = test_conn();
    let a1 = insert_artist(&mut c, "A1");
    let a2 = insert_artist(&mut c, "A2");
    insert_song(&mut c, "Same Song", &a1);
    insert_song(&mut c, "same song", &a2);
    insert_song(&mut c, "Unique Song", &a1);
    let r = commands::cmd_duplicates(&mut c, "song").unwrap();
    let dups = r["duplicates"].as_array().unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0]["name"], "same song");
    assert_eq!(dups[0]["records"].as_array().unwrap().len(), 2);
}

#[test]
fn test_duplicates_multiple_groups() {
    let mut c = test_conn();
    insert_artist(&mut c, "Alpha");
    insert_artist(&mut c, "ALPHA");
    insert_artist(&mut c, "Beta");
    insert_artist(&mut c, "BETA");
    let r = commands::cmd_duplicates(&mut c, "artist").unwrap();
    let dups = r["duplicates"].as_array().unwrap();
    assert_eq!(dups.len(), 2);
}
