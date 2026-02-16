pub mod db;
mod error;
mod handlers;
mod models;

use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

pub use db::{init_db, DbConnection};

pub fn create_app(db: DbConnection) -> Router {
    Router::new()
        .route("/{table}", get(handlers::read_by_id).post(handlers::insert))
        .layer(TraceLayer::new_for_http())
        .with_state(db)
}
