use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use agent_runtime::scorer::ScoreInput;
use crate::db;
use crate::models::{
    RunDetail, RunListQuery, RunListResponse, RunRecord, ToolCallRecord, UploadRequest,
};
use crate::AppState;

/// POST /api/runs — upload a new eval run
pub async fn upload_run(
    State(app): State<AppState>,
    Json(payload): Json<UploadRequest>,
) -> Result<Json<RunDetail>, (StatusCode, String)> {
    let pool = app.pool.clone();
    let scorer = app.scorer.clone();

    let run_id = payload
        .run_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let created_at = chrono::Utc::now().to_rfc3339();

    let result = scorer.score(&ScoreInput {
        events_json: &payload.events_json,
        status: &payload.status,
        total_tokens: payload.total_tokens,
        total_duration_ms: payload.total_duration_ms,
    });
    let safety_score = result.score;
    let alerts = result.alerts;

    let record = RunRecord {
        run_id,
        task_name: payload.task_name,
        created_at,
        model: payload.model,
        system_prompt: payload.system_prompt,
        max_turns: payload.max_turns,
        status: payload.status,
        total_turns: payload.total_turns,
        total_tokens: payload.total_tokens,
        total_duration_ms: payload.total_duration_ms,
        safety_score,
        events_json: payload.events_json,
    };

    let rid = record.run_id.clone();

    db::insert_run(&pool, &record).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to insert run: {}", e),
        )
    })?;

    db::insert_alerts(&pool, &rid, &alerts).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to insert alerts: {}", e),
        )
    })?;

    let detail = RunDetail {
        run_id: record.run_id,
        task_name: record.task_name,
        created_at: record.created_at,
        model: record.model,
        system_prompt: record.system_prompt,
        max_turns: record.max_turns,
        status: record.status,
        total_turns: record.total_turns,
        total_tokens: record.total_tokens,
        total_duration_ms: record.total_duration_ms,
        safety_score: record.safety_score,
        tool_calls: extract_tool_calls_from_json(&record.events_json),
        alerts,
        events_json: record.events_json,
    };

    Ok(Json(detail))
}

/// GET /api/runs — list runs with optional filtering and pagination
pub async fn list_runs(
    State(app): State<AppState>,
    Query(query): Query<RunListQuery>,
) -> Result<Json<RunListResponse>, (StatusCode, String)> {
    let pool = app.pool.clone();
    let (runs, total) = db::list_runs(&pool, &query).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to list runs: {}", e),
        )
    })?;

    Ok(Json(RunListResponse { runs, total }))
}

/// GET /api/runs/:id — get a single run with full detail
pub async fn get_run_detail(
    State(app): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunDetail>, (StatusCode, String)> {
    let pool = app.pool.clone();
    db::get_run_detail(&pool, &run_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get run detail: {}", e),
            )
        })?
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("run '{}' not found", run_id)))
}

/// Extract tool calls from an events JSON string.
fn extract_tool_calls_from_json(events_json: &str) -> Vec<ToolCallRecord> {
    let events: Vec<serde_json::Value> = serde_json::from_str(events_json).unwrap_or_default();

    events
        .iter()
        .filter_map(|event| {
            let event_type = event.get("type")?.as_str()?;

            if event_type != "tool_call" && event_type != "tool_use" {
                return None;
            }

            let tool_name = event
                .get("tool_name")
                .or_else(|| event.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let arguments = event
                .get("arguments")
                .or_else(|| event.get("input"))
                .unwrap_or(&serde_json::Value::Null)
                .clone();

            let result = event
                .get("result")
                .or_else(|| event.get("output"))
                .map(|v| v.to_string())
                .unwrap_or_default();

            let blocked = event
                .get("blocked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let is_error = event
                .get("is_error")
                .or_else(|| event.get("error"))
                .map(|v| !v.is_null())
                .unwrap_or(false);

            let timestamp = event
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // 旧 trace 无 duration_ms 时保持 None，前端不显示耗时
            let duration_ms = event.get("duration_ms").and_then(|v| v.as_u64());

            Some(ToolCallRecord {
                tool_name,
                arguments,
                result,
                blocked,
                is_error,
                timestamp,
                duration_ms,
            })
        })
        .collect()
}
