use std::process::Command;

fn cargo_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jankenoboe"));
    cmd.env("JANKENOBOE_DB", ":memory:");
    cmd
}

/// Create a temp SQLite DB with schema for CLI integration tests that need a working DB.
fn create_temp_db() -> (tempfile::NamedTempFile, String) {
    let tmp = tempfile::NamedTempFile::new().expect("create temp file");
    let path = tmp.path().to_str().unwrap().to_string();
    let conn = rusqlite::Connection::open(&path).expect("open temp db");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    conn.execute_batch(include_str!("../docs/init-db.sql"))
        .unwrap();
    (tmp, path)
}

fn cargo_bin_with_db(db_path: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jankenoboe"));
    cmd.env("JANKENOBOE_DB", db_path);
    cmd
}

#[test]
fn test_cli_no_args_shows_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_jankenoboe"))
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage"));
}

#[test]
fn test_cli_get_invalid_table() {
    let output = cargo_bin()
        .args(["get", "bad_table", "some-id", "--fields", "id"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid table"));
}

#[test]
fn test_cli_search_invalid_table() {
    let output = cargo_bin()
        .args([
            "search",
            "bad_table",
            "--term",
            r#"{"name":{"value":"x"}}"#,
            "--fields",
            "id",
        ])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid table"));
}

#[test]
fn test_cli_duplicates_invalid_table() {
    let output = cargo_bin()
        .args(["duplicates", "bad_table"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid table"));
}

#[test]
fn test_cli_create_invalid_table() {
    let output = cargo_bin()
        .args(["create", "bad_table", "--data", r#"{"name":"x"}"#])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_update_not_found() {
    let output = cargo_bin()
        .args([
            "update",
            "artist",
            "nonexistent-id",
            "--data",
            r#"{"name":"x"}"#,
        ])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_cli_delete_not_found() {
    let output = cargo_bin()
        .args(["delete", "artist", "nonexistent-id"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_cli_learning_due_no_schema() {
    // :memory: has no schema, so learning-due fails with a DB error
    let output = cargo_bin()
        .args(["learning-due", "--limit", "5"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_cli_learning_batch_empty() {
    let output = cargo_bin()
        .args(["learning-batch", "--song-ids", ""])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn test_cli_bulk_reassign_invalid_args() {
    let output = cargo_bin()
        .args(["bulk-reassign"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
}

#[test]
fn test_cli_missing_db_env() {
    let output = Command::new(env!("CARGO_BIN_EXE_jankenoboe"))
        .env_remove("JANKENOBOE_DB")
        .args(["get", "artist", "some-id", "--fields", "id"])
        .output()
        .expect("failed to run binary");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("JANKENOBOE_DB"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_jankenoboe"))
        .args(["--version"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("jankenoboe"));
}

// === CLI SUCCESS PATHS (using temp DB with schema) ===

#[test]
fn test_cli_create_and_get_success() {
    let (_tmp, db_path) = create_temp_db();

    // Create an artist
    let output = cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"TestArtist"}"#])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");
    let id = result["id"].as_str().unwrap();
    assert!(!id.is_empty());

    // Get the artist back
    let output2 = cargo_bin_with_db(&db_path)
        .args(["get", "artist", id, "--fields", "id,name"])
        .output()
        .expect("failed to run binary");
    assert!(output2.status.success());
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let result2: serde_json::Value = serde_json::from_str(&stdout2).expect("parse json");
    assert_eq!(result2["results"][0]["name"], "TestArtist");
}

#[test]
fn test_cli_duplicates_success() {
    let (_tmp, db_path) = create_temp_db();

    // Create two artists with no duplicates
    cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"A1"}"#])
        .output()
        .unwrap();
    cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"A2"}"#])
        .output()
        .unwrap();

    let output = cargo_bin_with_db(&db_path)
        .args(["duplicates", "artist"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");
    assert_eq!(result["duplicates"].as_array().unwrap().len(), 0);
}

#[test]
fn test_cli_search_success() {
    let (_tmp, db_path) = create_temp_db();

    cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"SearchMe"}"#])
        .output()
        .unwrap();

    let output = cargo_bin_with_db(&db_path)
        .args([
            "search",
            "artist",
            "--term",
            r#"{"name":{"value":"SearchMe"}}"#,
            "--fields",
            "id,name",
        ])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");
    assert_eq!(result["results"].as_array().unwrap().len(), 1);
}

#[test]
fn test_cli_learning_due_success() {
    let (_tmp, db_path) = create_temp_db();

    let output = cargo_bin_with_db(&db_path)
        .args(["learning-due", "--limit", "10"])
        .output()
        .expect("failed to run binary");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("parse json");
    assert_eq!(result["count"], 0);
    assert_eq!(result["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_cli_update_and_delete_success() {
    let (_tmp, db_path) = create_temp_db();

    // Create
    let out = cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"Old"}"#])
        .output()
        .unwrap();
    let result: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    let id = result["id"].as_str().unwrap().to_string();

    // Update
    let out2 = cargo_bin_with_db(&db_path)
        .args(["update", "artist", &id, "--data", r#"{"name":"New"}"#])
        .output()
        .unwrap();
    assert!(out2.status.success());
    let stdout2 = String::from_utf8_lossy(&out2.stdout);
    assert!(stdout2.contains("updated"));

    // Delete
    let out3 = cargo_bin_with_db(&db_path)
        .args(["delete", "artist", &id])
        .output()
        .unwrap();
    assert!(out3.status.success());
    let stdout3 = String::from_utf8_lossy(&out3.stdout);
    assert!(stdout3.contains("deleted"));
}

#[test]
fn test_cli_bulk_reassign_success() {
    let (_tmp, db_path) = create_temp_db();

    // Create two artists
    let out_a1 = cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"A1"}"#])
        .output()
        .unwrap();
    let a1: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_a1.stdout)).unwrap();
    let a1_id = a1["id"].as_str().unwrap().to_string();

    let out_a2 = cargo_bin_with_db(&db_path)
        .args(["create", "artist", "--data", r#"{"name":"A2"}"#])
        .output()
        .unwrap();
    let a2: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_a2.stdout)).unwrap();
    let a2_id = a2["id"].as_str().unwrap().to_string();

    // Create a song under A1
    let out_s = cargo_bin_with_db(&db_path)
        .args([
            "create",
            "song",
            "--data",
            &format!(r#"{{"name":"S1","artist_id":"{a1_id}"}}"#),
        ])
        .output()
        .unwrap();
    let s: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_s.stdout)).unwrap();
    let s_id = s["id"].as_str().unwrap().to_string();

    // Bulk reassign by artist
    let out_r = cargo_bin_with_db(&db_path)
        .args([
            "bulk-reassign",
            "--from-artist-id",
            &a1_id,
            "--to-artist-id",
            &a2_id,
        ])
        .output()
        .unwrap();
    assert!(out_r.status.success());
    let stdout_r = String::from_utf8_lossy(&out_r.stdout);
    let result_r: serde_json::Value = serde_json::from_str(&stdout_r).unwrap();
    assert_eq!(result_r["reassigned_count"], 1);

    // Verify song is now under A2
    let out_g = cargo_bin_with_db(&db_path)
        .args(["get", "song", &s_id, "--fields", "artist_id"])
        .output()
        .unwrap();
    let result_g: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&out_g.stdout)).unwrap();
    assert_eq!(result_g["results"][0]["artist_id"], a2_id);
}
