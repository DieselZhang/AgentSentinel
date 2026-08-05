pub mod db;
pub mod models;
pub mod routes;
pub mod scoring;

use agent_runtime::scorer::SafetyScorer;
use axum::{routing::get, Router};
use std::sync::Arc;

/// Shared application state: the database pool plus the pluggable safety scorer.
///
/// The scorer is the third audit interface (see `agent_runtime::audit`);
/// injecting it here keeps the eval-server routes decoupled from any concrete
/// scoring implementation.
#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
    pub scorer: Arc<dyn SafetyScorer>,
}

/// Build the application router for the given database pool and safety scorer.
///
/// More-specific routes MUST be registered before parameterised ones so
/// that e.g. /api/runs/compare is not captured by /api/runs/:id.
pub fn build_app(pool: db::DbPool, scorer: Arc<dyn SafetyScorer>) -> Router {
    let state = AppState { pool, scorer };
    Router::new()
        // GET /api/runs/compare?ids=id1,id2
        .route("/api/runs/compare", get(routes::compare::compare_runs))
        // GET /api/runs/:id/report
        .route("/api/runs/{id}/report", get(routes::report::get_report))
        // GET /api/runs/:id
        .route("/api/runs/{id}", get(routes::runs::get_run_detail))
        // GET /api/runs  +  POST /api/runs
        .route(
            "/api/runs",
            get(routes::runs::list_runs).post(routes::runs::upload_run),
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}
