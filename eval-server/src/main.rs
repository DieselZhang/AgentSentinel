use std::sync::{Arc, Mutex};

use agent_runtime::scorer::SafetyScorer;
use eval_server::db;
use eval_server::scoring::RuleBasedSafetyScorer;
use rusqlite::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database
    let conn = Connection::open("eval.db")?;
    let pool: db::DbPool = Arc::new(Mutex::new(conn));
    db::init_db(&pool)?;

    // Build the application router with the default rule-based scorer.
    let scorer: Arc<dyn SafetyScorer> = Arc::new(RuleBasedSafetyScorer);
    let app = eval_server::build_app(pool, scorer);

    // Bind and serve
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;
    println!("AgentSentinel eval-server listening on http://127.0.0.1:3001");

    axum::serve(listener, app).await?;

    Ok(())
}
