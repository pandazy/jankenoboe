use rusqlite::Connection;

/// Open a database connection.
///
/// Reads the database path from the `JANKENOBOE_DB` environment variable.
/// The database file must already exist with the correct schema
/// (see `docs/init-db.sql`).
pub fn open_connection() -> Result<Connection, crate::error::AppError> {
    let db_path = std::env::var("JANKENOBOE_DB").map_err(|_| {
        crate::error::AppError::InvalidParameter(
            "JANKENOBOE_DB environment variable is not set".to_string(),
        )
    })?;
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// Open an in-memory database and initialize the schema.
/// Used for testing.
#[cfg(test)]
pub fn open_test_connection() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to open in-memory database");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("Failed to enable foreign keys");
    let schema = include_str!("../docs/init-db.sql");
    conn.execute_batch(schema)
        .expect("Failed to initialize schema");
    conn
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_connection_missing_env() {
        // Temporarily remove the env var to test the error path
        let original = std::env::var("JANKENOBOE_DB").ok();
        // SAFETY: test runs single-threaded; env var is restored immediately after
        unsafe { std::env::remove_var("JANKENOBOE_DB") };
        let result = open_connection();
        // Restore the env var
        if let Some(val) = original {
            unsafe { std::env::set_var("JANKENOBOE_DB", val) };
        }
        assert_eq!(
            result.unwrap_err().to_string(),
            "JANKENOBOE_DB environment variable is not set"
        );
    }

    #[test]
    fn test_open_connection_with_memory_db() {
        let original = std::env::var("JANKENOBOE_DB").ok();
        // SAFETY: test runs single-threaded; env var is restored immediately after
        unsafe { std::env::set_var("JANKENOBOE_DB", ":memory:") };
        let result = open_connection();
        // Restore the env var
        match original {
            Some(val) => unsafe { std::env::set_var("JANKENOBOE_DB", val) },
            None => unsafe { std::env::remove_var("JANKENOBOE_DB") },
        }
        assert!(matches!(result, Ok(_)));
    }
}
