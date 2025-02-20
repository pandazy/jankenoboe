mod easing;
mod err;
mod learning;
mod store_handlers;
mod summary;
mod utils;

use store_handlers::{
    handle_schema, handle_store_create, handle_store_delete, handle_store_peer, handle_store_read,
    handle_store_update,
};
use utils::get_schema_family;

use jankenstore::sqlite::schema::SchemaFamily;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};
use hyper::{header, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use std::{env, sync::Arc};

const HTTP_LIST: [&str; 3] = [
    "http://localhost:3000",
    "http://localhost:5173",
    "http://localhost:5174",
];

const DEFAULT_DB_PATH: &str = "datasource.db";

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
        .route("/due_learning", get(learning::handle_due_learning))
        .route(
            "/level_to/{learning_id}/{level}",
            patch(learning::handle_level_up),
        )
        .route(
            "/total_due_learning",
            get(summary::handle_total_due_learning),
        )
        .route("/all_learning", get(learning::handle_all_learning))
        .route("/summary", get(summary::handle_summary))
        .with_state(handler_state)
        .layer(
            CorsLayer::new()
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                ])
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
