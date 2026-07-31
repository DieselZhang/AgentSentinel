use std::sync::{Arc, Mutex};

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use eval_server::{build_app, db};
use http_body_util::BodyExt;
use rusqlite::Connection;
use serde_json::{json, Value};
use tower::ServiceExt;

/// Create a fresh app backed by an in-memory SQLite database. Each test uses
/// its own pool because `ServiceExt::oneshot` consumes the router (we clone
/// the router per request).
fn test_app() -> Router {
    let conn = Connection::open_in_memory().unwrap();
    let pool: db::DbPool = Arc::new(Mutex::new(conn));
    db::init_db(&pool).unwrap();
    build_app(pool)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

fn post(body: &Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/runs")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// One valid `tool_call` event to embed in `events_json`.
fn sample_events() -> Value {
    json!([{
        "type": "tool_call",
        "tool_name": "read_file",
        "arguments": {"path": "/tmp/notes.txt"},
        "result": "file contents"
    }])
}

/// Upload a run and return its `run_id`.
async fn upload_run(app: &Router, task_name: &str) -> String {
    let payload = json!({
        "task_name": task_name,
        "model": "test-model",
        "system_prompt": "You are a helpful test agent.",
        "max_turns": 5,
        "events_json": sample_events().to_string(),
        "status": "success",
        "total_turns": 1,
        "total_tokens": 120,
        "total_duration_ms": 800,
    });
    let (status, body) = send(app, post(&payload)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "POST /api/runs should succeed; body: {}",
        body
    );
    let v: Value = serde_json::from_str(&body).unwrap();
    v["run_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_upload_and_list() {
    let app = test_app();
    let run_id = upload_run(&app, "demo").await;

    let (status, body) = send(&app, get("/api/runs")).await;
    assert_eq!(status, StatusCode::OK);
    let v: Value = serde_json::from_str(&body).unwrap();
    let ids: Vec<String> = v["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["run_id"].as_str().unwrap().to_string())
        .collect();
    assert!(
        ids.contains(&run_id),
        "list should contain uploaded run {}; got {:?}",
        run_id,
        ids
    );
}

#[tokio::test]
async fn test_get_run_detail() {
    let app = test_app();
    let run_id = upload_run(&app, "detail-task").await;

    let (status, body) = send(&app, get(&format!("/api/runs/{}", run_id))).await;
    assert_eq!(status, StatusCode::OK);
    let v: Value = serde_json::from_str(&body).unwrap();
    assert!(
        v["safety_score"].is_number(),
        "safety_score must be numeric; body: {}",
        body
    );
    let tool_calls = v["tool_calls"].as_array().unwrap();
    assert_eq!(tool_calls.len(), 1, "one tool_call event expected");
    assert_eq!(tool_calls[0]["tool_name"], "read_file");
}

#[tokio::test]
async fn test_compare_two_runs() {
    let app = test_app();
    let id1 = upload_run(&app, "compare-a").await;
    let id2 = upload_run(&app, "compare-b").await;

    let uri = format!("/api/runs/compare?ids={},{}", id1, id2);
    let (status, body) = send(&app, get(&uri)).await;
    assert_eq!(status, StatusCode::OK);
    let v: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["runs"].as_array().unwrap().len(), 2);
    assert!(
        v["comparison"].is_object(),
        "comparison field must exist; body: {}",
        body
    );
}

#[tokio::test]
async fn test_get_report() {
    let app = test_app();
    let run_id = upload_run(&app, "report-task").await;

    let (status, body) = send(&app, get(&format!("/api/runs/{}/report", run_id))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body.contains("# Run Report"),
        "report should be markdown; got: {}",
        &body[..body.len().min(200)]
    );
}

#[tokio::test]
async fn test_upload_missing_returns_404() {
    let app = test_app();
    let (status, body) = send(&app, get("/api/runs/nonexistent")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(
        body.contains("not found"),
        "expected not-found message; got: {}",
        body
    );
}
