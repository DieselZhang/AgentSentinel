mod db;
mod models;
mod routes;
mod scoring;

use std::sync::{Arc, Mutex};

use axum::{routing::get, Router};
use rusqlite::Connection;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let conn = Connection::open("eval.db")?;
    let pool: db::DbPool = Arc::new(Mutex::new(conn));
    db::init_db(&pool)?;

    // Build the application router.
    // More-specific routes MUST be registered before parameterised ones so
    // that e.g. /api/runs/compare is not captured by /api/runs/:id.
    let app = Router::new()
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
        .layer(CorsLayer::permissive())
        .with_state(pool);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    println!("AgentSentinel eval-server listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await?;

    Ok(())
}
