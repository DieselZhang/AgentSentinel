use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::models::{RunDetail, RunListQuery, RunRecord, RunSummary, SafetyAlert, ToolCallRecord};

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init_db(pool: &DbPool) -> Result<()> {
    let conn = pool.lock().unwrap();

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS runs (
            run_id TEXT PRIMARY KEY,
            task_name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            model TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            max_turns INTEGER NOT NULL,
            status TEXT NOT NULL,
            total_turns INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            total_duration_ms INTEGER NOT NULL,
            safety_score INTEGER NOT NULL DEFAULT 0,
            events_json TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS safety_alerts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id TEXT NOT NULL,
            severity TEXT NOT NULL,
            message TEXT NOT NULL,
            event_index INTEGER NOT NULL,
            FOREIGN KEY (run_id) REFERENCES runs(run_id) ON DELETE CASCADE
        );",
    )
    .context("failed to initialize database tables")?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

    Ok(())
}

pub fn insert_run(pool: &DbPool, run: &RunRecord) -> Result<()> {
    let conn = pool.lock().unwrap();

    conn.execute(
        "INSERT INTO runs (run_id, task_name, created_at, model, system_prompt, max_turns,
         status, total_turns, total_tokens, total_duration_ms, safety_score, events_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            run.run_id,
            run.task_name,
            run.created_at,
            run.model,
            run.system_prompt,
            run.max_turns,
            run.status,
            run.total_turns,
            run.total_tokens,
            run.total_duration_ms,
            run.safety_score,
            run.events_json,
        ],
    )
    .context("failed to insert run")?;

    Ok(())
}

pub fn insert_alerts(pool: &DbPool, run_id: &str, alerts: &[SafetyAlert]) -> Result<()> {
    let conn = pool.lock().unwrap();

    for alert in alerts {
        conn.execute(
            "INSERT INTO safety_alerts (run_id, severity, message, event_index)
             VALUES (?1, ?2, ?3, ?4)",
            params![run_id, alert.severity, alert.message, alert.event_index],
        )?;
    }

    Ok(())
}

pub fn list_runs(pool: &DbPool, query: &RunListQuery) -> Result<(Vec<RunSummary>, usize)> {
    let conn = pool.lock().unwrap();

    let mut where_clauses: Vec<String> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(ref task_name) = query.task_name {
        let idx = param_values.len() + 1;
        where_clauses.push(format!("task_name = ?{}", idx));
        param_values.push(Box::new(task_name.clone()));
    }

    if let Some(min_score) = query.min_score {
        let idx = param_values.len() + 1;
        where_clauses.push(format!("safety_score >= ?{}", idx));
        param_values.push(Box::new(min_score));
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Count total matching rows
    let count_sql = format!("SELECT COUNT(*) FROM runs {}", where_sql);
    let total: usize = conn
        .query_row(
            &count_sql,
            rusqlite::params_from_iter(param_values.iter().map(|p| p.as_ref())),
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Query with pagination
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let limit_idx = param_values.len() + 1;
    let offset_idx = param_values.len() + 2;

    let query_sql = format!(
        "SELECT run_id, task_name, created_at, model, status, safety_score, total_turns,
         total_duration_ms FROM runs {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        where_sql, limit_idx, offset_idx,
    );

    param_values.push(Box::new(limit as i64));
    param_values.push(Box::new(offset as i64));

    let mut stmt = conn.prepare(&query_sql)?;
    let rows = stmt.query_map(
        rusqlite::params_from_iter(param_values.iter().map(|p| p.as_ref())),
        |row| {
            Ok(RunSummary {
                run_id: row.get(0)?,
                task_name: row.get(1)?,
                created_at: row.get(2)?,
                model: row.get(3)?,
                status: row.get(4)?,
                safety_score: row.get(5)?,
                total_turns: row.get(6)?,
                total_duration_ms: row.get(7)?,
            })
        },
    )?;

    let mut runs: Vec<RunSummary> = Vec::new();
    for row in rows {
        runs.push(row?);
    }

    Ok((runs, total))
}

pub fn get_run_detail(pool: &DbPool, run_id: &str) -> Result<Option<RunDetail>> {
    let conn = pool.lock().unwrap();

    let run = conn.query_row(
        "SELECT run_id, task_name, created_at, model, system_prompt, max_turns,
         status, total_turns, total_tokens, total_duration_ms, safety_score, events_json
         FROM runs WHERE run_id = ?1",
        params![run_id],
        |row| {
            Ok(RunDetail {
                run_id: row.get(0)?,
                task_name: row.get(1)?,
                created_at: row.get(2)?,
                model: row.get(3)?,
                system_prompt: row.get(4)?,
                max_turns: row.get(5)?,
                status: row.get(6)?,
                total_turns: row.get(7)?,
                total_tokens: row.get(8)?,
                total_duration_ms: row.get(9)?,
                safety_score: row.get(10)?,
                tool_calls: Vec::new(),
                alerts: Vec::new(),
                events_json: row.get(11)?,
            })
        },
    );

    let mut detail = match run {
        Ok(d) => d,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(e) => return Err(e.into()),
    };

    // Parse tool calls from events_json
    detail.tool_calls = extract_tool_calls(&detail.events_json);

    // Fetch alerts from safety_alerts table
    let mut stmt = conn.prepare(
        "SELECT severity, message, event_index FROM safety_alerts WHERE run_id = ?1 ORDER BY event_index",
    )?;
    let alerts: Vec<SafetyAlert> = stmt
        .query_map(params![run_id], |row| {
            Ok(SafetyAlert {
                severity: row.get(0)?,
                message: row.get(1)?,
                event_index: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    detail.alerts = alerts;

    Ok(Some(detail))
}

fn extract_tool_calls(events_json: &str) -> Vec<ToolCallRecord> {
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

            Some(ToolCallRecord {
                tool_name,
                arguments,
                result,
                blocked,
                is_error,
                timestamp,
            })
        })
        .collect()
}
