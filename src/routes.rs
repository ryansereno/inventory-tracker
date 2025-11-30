use axum::{Router, routing::{get, post}};
use crate::handlers;

pub fn app_router() -> Router {
    Router::new()
        .route("/", get(handlers::show_form))
        .route("/ingest", post(handlers::ingest))
}
