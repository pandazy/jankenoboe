use crate::{
    easing, err,
    store_handlers::QueryRequest,
    utils::{get_db_conn, get_timestamp},
    HandlerState,
};

use jankenstore::{
    action::{CreateOp, ReadOp, UpdateOp},
    sqlite::{
        basics::{CountConfig, FetchConfig},
        read::{all, count},
        shift::{val::v_txt, val_to_json},
    },
};

use axum::{
    extract::{Path, Query, State},
    Json,
};
use hyper::StatusCode;
use serde_json::{from_value, json, Value};
use uuid::Uuid;

use std::sync::Arc;

/// customized handlers
pub async fn handle_try_to_learn(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Path(song_id): Path<String>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let conn = get_db_conn(db_path)?;

    let where_config = ("song_id=? and graduated=0", vec![v_txt(&song_id)]);
    let learning_count = count(
        &conn,
        schema_family,
        "learning",
        Some(CountConfig {
            where_config: Some((where_config.0, &where_config.1)),
            ..Default::default()
        }),
    )?;
    if learning_count > 0 {
        return Err(anyhow::anyhow!(err::http_err_msg(
            "You are already learning this song",
            StatusCode::BAD_REQUEST
        ))
        .into());
    }

    let wehere_config = ("id=?", vec![v_txt(&song_id)]);
    let existing_song_count = count(
        &conn,
        schema_family,
        "song",
        Some(CountConfig {
            where_config: Some((wehere_config.0, &wehere_config.1)),
            ..Default::default()
        }),
    )?;
    if existing_song_count == 0 {
        return Err(
            anyhow::anyhow!(err::http_err_msg("song not found", StatusCode::BAD_REQUEST)).into(),
        );
    }

    let mut easing = easing::new_easing_map(20)
        .values()
        .copied()
        .collect::<Vec<u16>>();
    easing.sort();
    let new_id = Uuid::new_v4().to_string();
    let time = get_timestamp();
    let create_op_cmd = json!({
        "Create": [
            "learning",
            {
                "id": new_id,
                "song_id": song_id,
                "level_up_path": easing,
                "updated_at": time,
                "created_at": time
            }
        ]
    });
    let create_op: CreateOp = from_value(create_op_cmd)?;
    create_op.run(&conn, schema_family)?;
    let read_op: ReadOp = from_value(json!({
        "ByPk": {
            "src": "learning",
            "keys": [new_id]
        }
    }))?;

    let (records, total) = read_op.run(&conn, schema_family, None)?;

    Ok(Json(json!({
        "records": records,
        "total": total
    })))
}

pub async fn handle_due_learning(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Query(query_params): Query<QueryRequest>,
) -> Result<Json<Value>, err::AppError> {
    let QueryRequest { limit, offset, .. } = query_params;
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let conn = get_db_conn(db_path)?;

    let level1_due = "level=0 and cast(strftime('%s', 'now') as INTEGER) >= (updated_at + 300)";
    let level2_due = "(json_extract(level_up_path, '$['||level||']')*24*60*60  + last_level_up_at) <= cast(strftime('%s', 'now') as INTEGER)";
    let where_cond = format!("graduated=0 and (({}) or ({})) ", level1_due, level2_due);
    let (records, total) = all(
        &conn,
        schema_family,
        "learning",
        Some(FetchConfig {
            where_config: Some((where_cond.as_str(), &[])),
            order_by: Some("level, last_level_up_at"),
            limit,
            offset,
            ..Default::default()
        }),
        false,
    )?;
    let mut json_records: Vec<Value> = Vec::new();
    for record in records {
        let json_record = val_to_json(&record)?;
        json_records.push(json_record);
    }

    Ok(Json(json!({
        "records": json_records,
        "total": total
    })))
}

pub async fn handle_level_up(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Path((learning_id, level)): Path<(String, u8)>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let conn = get_db_conn(db_path)?;

    let update_op_cmd = json!({
        "Update": [
            {
                "src": "learning",
                "keys": [learning_id],
            },
            {
                "level": level,
                "last_level_up_at": get_timestamp(),
                "updated_at": get_timestamp()
            }
        ]
    });
    let update_op: UpdateOp = from_value(update_op_cmd)?;
    update_op.run(&conn, schema_family)?;

    Ok(Json(json!({
        "message": "Level up successfully"
    })))
}
