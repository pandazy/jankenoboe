use crate::{easing, err, get_db_conn, get_timestamp};

use jankenstore::{
    action::{payload::ParsableOp, CreateOp, ReadOp},
    sqlite::{basics::CountConfig, read::count, schema, shift::val::v_txt},
};

use axum::{
    extract::{Path, State},
    Json,
};
use hyper::StatusCode;
use serde_json::{from_value, json, Value};
use uuid::Uuid;

use std::sync::Arc;

pub async fn handle_try_to_learn(
    State(schema_family): State<Arc<schema::SchemaFamily>>,
    Path(song_id): Path<String>,
) -> Result<Json<Value>, err::AppError> {
    let conn = get_db_conn()?;

    let where_config = ("song_id=? and graduated=0", vec![v_txt(&song_id)]);
    let learning_count = count(
        &conn,
        &schema_family,
        "learning",
        Some(CountConfig {
            where_config: Some((where_config.0, &where_config.1)),
            ..Default::default()
        }),
    )?;
    if learning_count > 0 {
        return Err(anyhow::anyhow!(err::http_err_msg(
            "already learned",
            StatusCode::BAD_REQUEST
        ))
        .into());
    }

    let wehere_config = ("id=?", vec![v_txt(&song_id)]);
    let existing_song_count = count(
        &conn,
        &schema_family,
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
    create_op.run(&conn, &schema_family)?;
    let read_op = ReadOp::from_str(&format!(
        r#"{{
          "ByPk": {{
              "src": "learning",
              "keys": ["{}"]
          }}
      }}"#,
        new_id
    ))?;
    let results = read_op.run(&conn, &schema_family, None)?;

    Ok(Json(json!(results[0])))
}
