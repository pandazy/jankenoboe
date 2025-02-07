mod easing;
mod err;
mod learning;

use jankenstore::{
    action::{payload::ReadSrc, CreateOp, DelOp, PeerOp, ReadOp, UpdateOp},
    sqlite::{
        basics::FetchConfig,
        schema::{self, fetch_schema_family, Schema, SchemaFamily},
        shift::val::{v_int, v_txt},
    },
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use hyper::{header, Method};
use rusqlite::{types, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use tower_http::cors::{AllowOrigin, CorsLayer};
use uuid::Uuid;

use std::{collections::HashMap, env, sync::Arc, time::SystemTime};

const HTTP_LIST: [&str; 2] = ["http://localhost:3000", "http://localhost:5173"];
const DEFAULT_DB_PATH: &str = "datasource.db";

fn get_db_conn(path: &str) -> anyhow::Result<Connection> {
    let conn = Connection::open(path)?;
    Ok(conn)
}

fn get_schema_family(db_path: &str) -> anyhow::Result<schema::SchemaFamily> {
    let conn = get_db_conn(db_path)?;
    let schema_family = fetch_schema_family(&conn, &[], "", "")?;
    Ok(schema_family)
}

///
/// The pagination configuration for fetching records
#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub order_by: Option<String>,
}

#[derive(Debug)]
pub struct HandlerState {
    pub schema_family: SchemaFamily,
    pub db_path: String,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());

    println!("DB_PATH: {}", db_path);

    let schema_family_result = get_schema_family(&db_path);
    if let Err(e) = schema_family_result {
        eprintln!("Error: {}", e);
        return;
    }

    let handler_state = Arc::new(HandlerState {
        schema_family: schema_family_result.unwrap(),
        db_path,
    });

    // build our application with a route
    let app = Router::new()
        .route("/schema", get(handle_schema))
        .route("/store_read", get(handle_store_read))
        .route("/store_create", post(handle_store_create))
        .route("/store_update", put(handle_store_update))
        .route("/store_delete", delete(handle_store_delete))
        .route("/store_peer", patch(handle_store_peer))
        .route(
            "/try_to_learn/{song_id}",
            post(learning::handle_try_to_learn),
        )
        .with_state(handler_state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_origin(AllowOrigin::list(
                    HTTP_LIST
                        .iter()
                        .map(|x| x.parse().unwrap())
                        .collect::<Vec<_>>(),
                ))
                .allow_headers([header::CONTENT_TYPE]),
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn get_body(body: &str) -> Result<Value, err::AppError> {
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

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn set_timestamped_input(
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

async fn handle_schema(
    State(handle_state): State<Arc<HandlerState>>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState { schema_family, .. } = &*handle_state;
    Ok(Json(schema_family.json()?))
}

async fn handle_store_read(
    State(handler_state): State<Arc<HandlerState>>,
    Query(query): Query<Pagination>,
    body: String,
) -> Result<Json<Vec<Value>>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = &*handler_state;
    let body = get_body(&body)?;
    let Pagination {
        limit,
        offset,
        order_by,
    } = query;

    let conn = get_db_conn(db_path)?;
    let op: ReadOp = from_value(body)?;
    let where_config = ("status=0", vec![]);
    let fetch_cfg = FetchConfig {
        limit,
        offset,
        order_by: order_by.as_deref(),
        where_config: Some((where_config.0, &where_config.1)),
        ..Default::default()
    };
    let results = op.run(&conn, schema_family, Some(fetch_cfg))?;
    Ok(Json(results))
}

async fn handle_store_create(
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
    let results = read.run(&conn, schema_family, None)?;
    Ok(Json(json!(results[0])))
}

async fn handle_store_update(
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

async fn handle_store_delete(
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

async fn handle_store_peer(
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
