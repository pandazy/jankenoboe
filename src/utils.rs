use std::{collections::HashMap, time::SystemTime};

use hyper::StatusCode;
use jankenstore::sqlite::{
    schema::{self, fetch_schema_family, Schema},
    shift::val::v_int,
};

use rusqlite::{types, Connection};
use serde_json::Value;

use crate::err;

pub fn get_db_conn(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

pub fn get_schema_family(db_path: &str) -> anyhow::Result<schema::SchemaFamily> {
    let conn = get_db_conn(db_path)?;
    let schema_family = fetch_schema_family(&conn, &[], "", "")?;
    Ok(schema_family)
}

pub fn get_body(body: &str) -> Result<Value, err::AppError> {
    if body.is_empty() {
        return Err(anyhow::anyhow!(err::http_err_msg(
            "missing request body",
            StatusCode::BAD_REQUEST
        ))
        .into());
    }
    let body: Value = serde_json::from_str(body)?;
    Ok(body)
}

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn set_timestamped_input(
    schema: &Schema,
    input: &mut HashMap<String, types::Value>,
    timestamp_fields: &[&str],
) -> anyhow::Result<()> {
    for field in timestamp_fields {
        if schema.types.contains_key(*field) {
            input.insert(field.to_string(), v_int(get_timestamp().try_into()?));
        }
    }
    Ok(())
}
