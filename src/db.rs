use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

pub fn init_db() -> Result<DbConnection, rusqlite::Error> {
    let conn = Connection::open("store.db")?;

    // Create a sample table for testing
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL
        )",
        [],
    )?;

    // Insert sample data if table is empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute(
            "INSERT INTO users (id, name, email) VALUES ('1', 'Alice Smith', 'alice@example.com')",
            [],
        )?;
        conn.execute(
            "INSERT INTO users (id, name, email) VALUES ('2', 'Bob Jones', 'bob@example.com')",
            [],
        )?;
        conn.execute(
            "INSERT INTO users (id, name, email) VALUES ('3', 'Charlie Brown', 'charlie@example.com')",
            [],
        )?;
    }

    Ok(Arc::new(Mutex::new(conn)))
}
