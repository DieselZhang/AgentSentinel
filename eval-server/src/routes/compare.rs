use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use crate::db;
use crate::models::{CompareQuery, RunDetail};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct CompareResponse {
    pub runs: Vec<RunDetail>,
    pub comparison: CompareSummary,
}

#[derive(Debug, Serialize)]
pub struct CompareSummary {
    pub score_diff: i32,
    pub turns_diff: i32,
    pub tokens_diff: i64,
    pub duration_diff_ms: i64,
}

/// GET /api/runs/compare?ids=id1,id2 — compare two runs side by side
pub async fn compare_runs(
    State(app): State<AppState>,
    Query(query): Query<CompareQuery>,
) -> Result<Json<CompareResponse>, (StatusCode, String)> {
    let pool = app.pool.clone();

    let ids: Vec<&str> = query.ids.split(',').map(|s| s.trim()).collect();

    if ids.len() < 2 {
        return Err((
            StatusCode::BAD_REQUEST,
            "at least two run IDs are required, comma-separated".to_string(),
        ));
    }

    let id1 = ids[0];
    let id2 = ids[1];

    let run1 = db::get_run_detail(&pool, id1)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get run '{}': {}", id1, e),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("run '{}' not found", id1)))?;

    let run2 = db::get_run_detail(&pool, id2)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get run '{}': {}", id2, e),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("run '{}' not found", id2)))?;

    let comparison = CompareSummary {
        score_diff: (run1.safety_score as i32) - (run2.safety_score as i32),
        turns_diff: (run1.total_turns as i32) - (run2.total_turns as i32),
        tokens_diff: (run1.total_tokens as i64) - (run2.total_tokens as i64),
        duration_diff_ms: (run1.total_duration_ms as i64) - (run2.total_duration_ms as i64),
    };

    Ok(Json(CompareResponse {
        runs: vec![run1, run2],
        comparison,
    }))
}
