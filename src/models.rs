use serde::{Deserialize, Serialize};

/// Query parameters for read_by_id endpoint
#[derive(Debug, Deserialize)]
pub struct ReadByIdQuery {
    pub id: String,
}

/// JSON body for read_by_id endpoint
#[derive(Debug, Deserialize)]
pub struct ReadByIdBody {
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadByIdResponse {
    pub results: Vec<serde_json::Value>,
}

/// JSON body for insert endpoint
#[derive(Debug, Deserialize)]
pub struct InsertBody {
    pub values: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct InsertResponse {
    pub id: String,
}
