use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};

use crate::db::{self, DbPool};

/// GET /api/runs/:id/report — export a run as a markdown report
pub async fn get_report(
    State(pool): State<DbPool>,
    Path(run_id): Path<String>,
) -> Result<Response, (StatusCode, String)> {
    let detail = db::get_run_detail(&pool, &run_id)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to get run detail: {}", e),
            )
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("run '{}' not found", run_id)))?;

    let markdown = generate_markdown(&detail);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"run-{}.md\"", detail.run_id),
        )
        .body(axum::body::Body::from(markdown))
        .unwrap())
}

fn generate_markdown(detail: &crate::models::RunDetail) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# Run Report: {}\n\n", detail.run_id));
    md.push_str(&format!("**Task**: {}\n\n", detail.task_name));
    md.push_str(&format!("**Model**: {}\n\n", detail.model));
    md.push_str(&format!("**Created**: {}\n\n", detail.created_at));

    // Summary table
    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Value |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!("| Status | {} |\n", detail.status));
    md.push_str(&format!(
        "| Safety Score | {}/100 |\n",
        detail.safety_score
    ));
    md.push_str(&format!("| Total Turns | {} |\n", detail.total_turns));
    md.push_str(&format!("| Max Turns | {} |\n", detail.max_turns));
    md.push_str(&format!("| Total Tokens | {} |\n", detail.total_tokens));
    md.push_str(&format!(
        "| Total Duration | {} ms |\n",
        detail.total_duration_ms
    ));
    md.push('\n');

    // System prompt
    md.push_str("## System Prompt\n\n");
    md.push_str("```\n");
    md.push_str(&detail.system_prompt);
    md.push_str("\n```\n\n");

    // Safety alerts
    md.push_str("## Safety Alerts\n\n");
    if detail.alerts.is_empty() {
        md.push_str("*No safety alerts.*\n\n");
    } else {
        md.push_str("| Severity | Event | Message |\n");
        md.push_str("|----------|-------|---------|\n");
        for alert in &detail.alerts {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                alert.severity, alert.event_index, alert.message
            ));
        }
        md.push('\n');
    }

    // Tool calls
    md.push_str("## Tool Calls\n\n");
    if detail.tool_calls.is_empty() {
        md.push_str("*No tool calls recorded.*\n\n");
    } else {
        for (i, tc) in detail.tool_calls.iter().enumerate() {
            md.push_str(&format!("### {}. {}\n\n", i + 1, tc.tool_name));
            md.push_str(&format!("- **Blocked**: {}\n", tc.blocked));
            md.push_str(&format!("- **Error**: {}\n", tc.is_error));
            md.push_str(&format!("- **Timestamp**: {}\n", tc.timestamp));
            md.push_str(&format!(
                "- **Arguments**: `{}`\n",
                serde_json::to_string_pretty(&tc.arguments).unwrap_or_default()
            ));
            if !tc.result.is_empty() {
                let truncated = if tc.result.len() > 500 {
                    format!("{}...(truncated)", &tc.result[..500])
                } else {
                    tc.result.clone()
                };
                md.push_str(&format!("- **Result**:\n```\n{}\n```\n", truncated));
            }
            md.push('\n');
        }
    }

    md
}
