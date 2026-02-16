use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub type DbConnection = Arc<Mutex<Connection>>;

/// Initialize the database connection.
///
/// Reads the database path from the `JANKENOBOE_DB` environment variable.
/// The database file must already exist with the correct schema
/// (see `docs/init-db.sql`).
///
/// # Panics
///
/// Panics if `JANKENOBOE_DB` is not set or the database cannot be opened.
pub fn init_db() -> Result<DbConnection, rusqlite::Error> {
    let db_path = std::env::var("JANKENOBOE_DB").unwrap_or_else(|_| {
        eprintln!("Error: JANKENOBOE_DB environment variable is not set.");
        eprintln!("Set it to the path of your SQLite database file, e.g.:");
        eprintln!("  export JANKENOBOE_DB=~/db/datasource.db");
        std::process::exit(1);
    });

    let conn = Connection::open(&db_path)?;
    Ok(Arc::new(Mutex::new(conn)))
}
