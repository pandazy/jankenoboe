use axum::{
    extract::{Path, Query, State},
    Json,
};
use jankensqlhub::QueryDefinitions;
use serde_json::json;

use crate::{
    db::DbConnection,
    error::AppError,
    models::{InsertBody, InsertResponse, ReadByIdBody, ReadByIdQuery, ReadByIdResponse},
};

pub async fn read_by_id(
    State(db): State<DbConnection>,
    Path(table): Path<String>,
    Query(params): Query<ReadByIdQuery>,
    Json(body): Json<ReadByIdBody>,
) -> Result<Json<ReadByIdResponse>, AppError> {
    if body.fields.is_empty() {
        return Err(AppError::InvalidParameter(
            "fields cannot be empty".to_string(),
        ));
    }

    // Define query using QueryDefinitions::from_json with ~[fields] comma_list parameter
    // Using enum to restrict allowed field names for SQL injection prevention
    let query_json = json!({
        "read_by_id": {
            "query": "SELECT ~[fields] FROM #[table] WHERE id = @id",
            "returns": "~[fields]",
            "args": {
                "table": {"enum": ["users"]},
                "id": {"type": "string"},
                "fields": {"enum": ["id", "name", "email", "created_at", "updated_at"]}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Failed to create query definitions: {}", e)))?;

    // Prepare arguments as JSON with fields as an array from body
    let request_params = json!({
        "table": table,
        "id": params.id,
        "fields": body.fields
    });

    // Execute query
    let mut conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("Lock error: {}", e)))?;

    let result =
        jankensqlhub::query_run_sqlite(&mut conn, &queries, "read_by_id", &request_params)?;

    Ok(Json(ReadByIdResponse {
        results: result.data,
    }))
}

pub async fn insert(
    State(db): State<DbConnection>,
    Path(table): Path<String>,
    Json(body): Json<InsertBody>,
) -> Result<Json<InsertResponse>, AppError> {
    if body.values.is_empty() {
        return Err(AppError::InvalidParameter(
            "values cannot be empty".to_string(),
        ));
    }

    // Generate a unique id for the insert
    let id = uuid::Uuid::new_v4().to_string();

    // Build field names and values arrays, prepending id
    let mut field_names: Vec<String> = vec!["id".to_string()];
    let mut field_values: Vec<serde_json::Value> = vec![json!(id)];

    for (key, value) in &body.values {
        field_names.push(key.clone());
        field_values.push(value.clone());
    }

    // Define query using QueryDefinitions::from_json
    // Using enum to restrict allowed field and table names for SQL injection prevention
    // :[values] creates parameter-bound placeholders (?, ?, ?) including parentheses
    let query_json = json!({
        "insert": {
            "query": "INSERT INTO #[table] (~[fields]) VALUES :[values]",
            "args": {
                "table": {"enum": ["users"]},
                "fields": {"enum": ["id", "name", "email", "created_at", "updated_at"]},
                "values": {"itemtype": "string"}
            }
        }
    });

    let queries = QueryDefinitions::from_json(query_json)
        .map_err(|e| AppError::Internal(format!("Failed to create query definitions: {}", e)))?;

    // Prepare arguments as JSON
    let request_params = json!({
        "table": table,
        "fields": field_names,
        "values": field_values
    });

    // Execute query
    let mut conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("Lock error: {}", e)))?;

    jankensqlhub::query_run_sqlite(&mut conn, &queries, "insert", &request_params)?;

    Ok(Json(InsertResponse { id }))
}
