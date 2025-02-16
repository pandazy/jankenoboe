use crate::{
    err,
    utils::{get_body, get_db_conn, set_timestamped_input},
    HandlerState,
};

use jankenstore::{
    action::{
        payload::{ParsableOp, ReadSrc},
        CreateOp, DelOp, PeerOp, ReadOp, UpdateOp,
    },
    sqlite::{basics::FetchConfig, shift::val::v_txt},
};

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Query, State},
    Json,
};
use hyper::StatusCode;
use rusqlite::types;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use uuid::Uuid;

///
/// The pagination configuration for fetching records
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order_by: Option<String>,
    pub op: Option<String>,
}

pub async fn handle_schema(
    State(handle_state): State<Arc<HandlerState>>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState { schema_family, .. } = &*handle_state;
    Ok(Json(schema_family.json()?))
}

pub async fn handle_store_read(
    State(handler_state): State<Arc<HandlerState>>,
    Query(query): Query<QueryRequest>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let QueryRequest {
        limit,
        offset,
        order_by,
        op,
    } = query;

    let op = op.unwrap_or_else(|| "".to_string());

    let conn = get_db_conn(db_path)?;
    let op = ReadOp::from_str(&op)?;
    let schema = schema_family.try_get_schema(op.src())?;
    let fetch_cfg = FetchConfig {
        limit,
        offset,
        order_by: order_by.as_deref(),
        where_config: if schema.types.contains_key("status") {
            Some(("status=0", &[]))
        } else {
            None
        },
        ..Default::default()
    };
    let (results, count) = op.run(&conn, schema_family, Some(fetch_cfg))?;
    Ok(Json(json!({
        "records": results,
        "total": count
    })))
}

pub async fn handle_store_create(
    State(handler_state): State<Arc<HandlerState>>,
    body: String,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let body = get_body(&body)?;
    let conn = get_db_conn(db_path)?;
    let create_op: CreateOp = from_value(body)?;
    let new_id = Uuid::new_v4().to_string();
    create_op.run_map(&conn, schema_family, |input, src| {
        let schema = schema_family.try_get_schema(src)?;
        let pk_type = schema
            .types
            .get(&schema.pk)
            .ok_or(anyhow::anyhow!(err::http_err_msg(
                "pk not found",
                StatusCode::BAD_REQUEST
            )))?;

        if !pk_type.eq(&types::Type::Text) {
            return Err(anyhow::anyhow!(err::http_err_msg(
                "pk must be text",
                StatusCode::BAD_REQUEST
            )));
        }
        let mut input = input.iter().fold(
            HashMap::from([(schema.pk.clone(), v_txt(&new_id))]),
            |mut acc, (k, v)| {
                acc.insert(k.clone(), v.clone());
                acc
            },
        );
        set_timestamped_input(schema, &mut input, &["created_at", "updated_at"])?;
        Ok(input)
    })?;

    let read: ReadOp = from_value(json!({
        "ByPk": {
            "src": create_op.src(),
            "keys": [new_id]
        }
    }))?;
    let (results, _) = read.run(&conn, schema_family, None)?;
    Ok(Json(json!(results[0])))
}

pub async fn handle_store_update(
    State(handler_state): State<Arc<HandlerState>>,
    body: String,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let body = get_body(&body)?;
    let conn = get_db_conn(db_path)?;
    let op: UpdateOp = from_value(body)?;
    op.run_map(&conn, schema_family, |input, src| {
        let schema = schema_family.try_get_schema(src)?;
        let mut input = input.clone();
        set_timestamped_input(schema, &mut input, &["updated_at"])?;
        Ok(input)
    })?;
    Ok(Json(json!({})))
}

pub async fn handle_store_delete(
    State(handler_state): State<Arc<HandlerState>>,
    body: String,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let body = get_body(&body)?;
    let conn = get_db_conn(db_path)?;
    let op: DelOp = from_value(body)?;
    op.run(&conn, schema_family, None)?;
    Ok(Json(json!({})))
}

pub async fn handle_store_peer(
    State(handler_state): State<Arc<HandlerState>>,
    body: String,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let body = get_body(&body)?;
    let conn = get_db_conn(db_path)?;
    let op: PeerOp = from_value(body)?;
    op.run(&conn, schema_family)?;
    Ok(Json(json!({})))
}
