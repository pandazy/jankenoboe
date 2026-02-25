use serde_json::Value;

/// Convert a serde_json::Value to a boxed rusqlite::ToSql.
pub fn json_value_to_sql(val: &Value) -> Box<dyn rusqlite::ToSql> {
    match val {
        Value::String(s) => Box::new(s.clone()),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Box::new(i)
            } else if let Some(f) = n.as_f64() {
                Box::new(f)
            } else {
                Box::new(n.to_string())
            }
        }
        Value::Bool(b) => Box::new(*b as i64),
        Value::Null => Box::new(rusqlite::types::Null),
        _ => Box::new(val.to_string()),
    }
}
