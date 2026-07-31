use std::sync::{Arc, Mutex};

use eval_server::db;
use rusqlite::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let conn = Connection::open("eval.db")?;
    let pool: db::DbPool = Arc::new(Mutex::new(conn));
    db::init_db(&pool)?;

    // Build the application router (see eval_server::build_app).
    let app = eval_server::build_app(pool);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    println!("AgentSentinel eval-server listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await?;

    Ok(())
}
