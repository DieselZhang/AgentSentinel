use clap::{Parser, Subcommand};
use agent_runtime::loop_::{run_agent, AgentConfig};
use agent_runtime::provider::{AnthropicProvider, DeepseekProvider, LlmProvider, OpenAIProvider};
use agent_runtime::tools::read::ReadFile;
use agent_runtime::tools::write::WriteFile;
use agent_runtime::tools::bash::Bash;
use agent_runtime::tool::Tool;
use agent_runtime::policy::{PermissionPolicy, DenyDangerous};
use agent_runtime::trace::InMemoryEmitter;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "agent-sentinel")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
enum Commands {
    Run { #[arg(long)] task: String, #[arg(long)] prompt: String, #[arg(long, default_value = "You are a helpful AI assistant.")] system: String, #[arg(long, default_value = "10")] max_turns: usize, #[arg(long, default_value = "anthropic")] provider: String },
    Upload { file: String, #[arg(long, default_value = "http://127.0.0.1:3001")] server: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { task, prompt, system, max_turns, provider } => {
            let (provider, model) = match provider.as_str() {
                "deepseek" | "ds" => (
                    Arc::new(DeepseekProvider::from_env()?) as Arc<dyn LlmProvider>,
                    std::env::var("DEEPSEEK_MODEL").unwrap_or_else(|_| "deepseek-chat".into()),
                ),
                "openai" | "oai" => (
                    Arc::new(OpenAIProvider::from_env()?) as Arc<dyn LlmProvider>,
                    std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into()),
                ),
                _ => (
                    Arc::new(AnthropicProvider::from_env()?) as Arc<dyn LlmProvider>,
                    std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-sonnet-4-5@20250929".into()),
                ),
            };
            let policy: Arc<dyn PermissionPolicy> = Arc::new(DenyDangerous::default());
            let tracer = Arc::new(InMemoryEmitter::new());
            let tools: Vec<Arc<dyn Tool>> = vec![Arc::new(ReadFile), Arc::new(WriteFile), Arc::new(Bash)];
            let config = AgentConfig { system_prompt: system, max_turns, tools, policy, provider, tracer: tracer.clone(), model };
            println!("Running: {}", task);
            let start = std::time::Instant::now();
            match run_agent(config, &task, &prompt).await {
                Ok((messages, events)) => {
                    let status = events.iter().rev().find_map(|e| match e { agent_runtime::types::AgentEvent::RunEnd{status} => Some(status.clone()), _ => None }).unwrap_or("unknown".into());
                    let turns = messages.iter().filter(|m| m.role == agent_runtime::types::Role::Assistant).count();
                    let now = chrono::Utc::now().to_rfc3339();
                    // 真实总耗时：写入 trace 的 run_end 事件，upload 时供评分使用
                    let duration_ms = start.elapsed().as_millis() as u64;
                    let trace_events = events_to_trace(&events, &now, duration_ms);
                    let fname = format!("traces/{}_{}.json", task, chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                    std::fs::create_dir_all("traces")?;
                    std::fs::write(&fname, &serde_json::to_string_pretty(&trace_events)?)?;
                    println!("Status: {}, Turns: {}, Duration: {}ms", status, turns, duration_ms);
                    println!("Trace: {}", fname);
                }
                Err(e) => eprintln!("Failed: {}", e),
            }
        }
        Commands::Upload { file, server } => {
            let content = std::fs::read_to_string(&file)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            // 用文件名（去扩展名）作 task_name，兼容相对/绝对路径
            let task = std::path::Path::new(&file)
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| file.clone());
            // 从 trace 的 run_end 事件提取状态作为评分依据（缺省 success）
            let status = json
                .as_array()
                .and_then(|events| events.iter().rev().find(|e| e["type"] == "run_end"))
                .and_then(|e| e["status"].as_str())
                .unwrap_or("success")
                .to_string();
            // 从 run_end 事件读取 cli run 时写入的真实总耗时（缺省 0，旧 trace 兼容）
            let total_duration_ms = json
                .as_array()
                .and_then(|events| events.iter().rev().find(|e| e["type"] == "run_end"))
                .and_then(|e| e["duration_ms"].as_u64())
                .unwrap_or(0);
            let c = reqwest::Client::new();
            let r = c.post(format!("{}/api/runs",server)).json(&serde_json::json!({
                "task_name":task,"model":"unknown","system_prompt":"","max_turns":10,
                "events_json":serde_json::to_string(&json).unwrap_or_default(),
                "status":status,"total_turns":1,"total_tokens":0,"total_duration_ms":total_duration_ms
            })).send().await?;
            if r.status().is_success() { let v: serde_json::Value = r.json().await?; println!("Uploaded! ID: {}", v["run_id"].as_str().unwrap_or("")); } else { eprintln!("Upload failed: {}", r.status()); }
        }
    }
    Ok(())
}

/// Convert runtime AgentEvents into the flat trace format the eval-server
/// consumes. `tool_call_start` + `tool_call_end` are merged into a single
/// `tool_call` event: the start records the tool name, the end carries the
/// outcome (result / blocked / arguments).
fn events_to_trace(
    events: &[agent_runtime::types::AgentEvent],
    now: &str,
    total_duration_ms: u64,
) -> Vec<serde_json::Value> {
    use agent_runtime::types::AgentEvent;
    use std::collections::HashMap;

    let mut tool_names: HashMap<String, String> = HashMap::new();
    let mut out: Vec<serde_json::Value> = Vec::new();

    for event in events {
        // tool_call_start 只记录 tool_name，等 tool_call_end 合成完整事件
        if let AgentEvent::ToolCallStart { tool_name, tool_call_id, .. } = event {
            tool_names.insert(tool_call_id.clone(), tool_name.clone());
            continue;
        }

        let (event_type, data) = match event {
            AgentEvent::TextDelta { text } => ("text_delta", serde_json::json!({ "text": text })),
            AgentEvent::ThinkingDelta { text } => ("thinking", serde_json::json!({ "text": text })),
            AgentEvent::ToolCallEnd { tool_call_id, result, is_error, blocked, duration_ms, arguments } => {
                let tool_name = tool_names
                    .remove(tool_call_id)
                    .unwrap_or_else(|| "unknown".to_string());
                (
                    "tool_call",
                    serde_json::json!({
                        "tool_name": tool_name,
                        "arguments": arguments,
                        "result": result,
                        "is_error": is_error,
                        "blocked": blocked,
                        "duration_ms": duration_ms,
                    }),
                )
            }
            AgentEvent::TurnStart { turn } => ("turn_start", serde_json::json!({ "turn": turn })),
            AgentEvent::TurnEnd { turn } => ("turn_end", serde_json::json!({ "turn": turn })),
            AgentEvent::RunStart { task_name } => {
                ("run_start", serde_json::json!({ "task_name": task_name }))
            }
            AgentEvent::RunEnd { status } => (
                "run_end",
                serde_json::json!({ "status": status, "duration_ms": total_duration_ms }),
            ),
            AgentEvent::Error { message } => ("error", serde_json::json!({ "message": message })),
            AgentEvent::ToolCallStart { .. } => unreachable!("handled above"),
        };

        // 扁平事件：timestamp + type + 其余字段
        let mut flat = serde_json::Map::new();
        flat.insert("timestamp".to_string(), serde_json::json!(now));
        flat.insert("type".to_string(), serde_json::json!(event_type));
        if let Some(obj) = data.as_object() {
            for (k, v) in obj {
                flat.insert(k.clone(), v.clone());
            }
        }
        out.push(serde_json::Value::Object(flat));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Commands;
    use clap::Parser;

    #[test]
    fn test_run_command_parsing() {
        let cli = super::Cli::try_parse_from([
            "agent-sentinel",
            "run",
            "--task",
            "demo",
            "--prompt",
            "hello",
            "--provider",
            "deepseek",
        ])
        .unwrap();

        match cli.command {
            Commands::Run {
                task,
                prompt,
                provider,
                ..
            } => {
                assert_eq!(task, "demo");
                assert_eq!(prompt, "hello");
                assert_eq!(provider, "deepseek");
            }
            _ => panic!("expected Run command"),
        }
    }

    #[test]
    fn test_openai_command_parsing() {
        let cli = super::Cli::try_parse_from([
            "agent-sentinel",
            "run",
            "--task",
            "demo",
            "--prompt",
            "hello",
            "--provider",
            "openai",
        ])
        .unwrap();

        match cli.command {
            Commands::Run { provider, .. } => assert_eq!(provider, "openai"),
            _ => panic!("expected Run command"),
        }
    }

    #[test]
    fn test_upload_command_parsing() {
        let cli = super::Cli::try_parse_from(["agent-sentinel", "upload", "trace.json"]).unwrap();

        match cli.command {
            Commands::Upload { file, .. } => assert_eq!(file, "trace.json"),
            _ => panic!("expected Upload command"),
        }
    }

    #[test]
    fn test_missing_required_arg_fails() {
        // `run` requires both --task and --prompt.
        assert!(super::Cli::try_parse_from(["agent-sentinel", "run"]).is_err());
    }

    #[test]
    fn test_upload_task_name_uses_file_stem() {
        // 相对路径
        let rel = std::path::Path::new("traces/e2e-demo.json");
        assert_eq!(rel.file_stem().unwrap().to_string_lossy(), "e2e-demo");
        // 绝对路径（脚本传入）同样只取文件名
        let abs = std::path::Path::new("/Users/me/project/traces/e2e-demo.json");
        assert_eq!(abs.file_stem().unwrap().to_string_lossy(), "e2e-demo");
    }

    #[test]
    fn test_events_to_trace_merges_tool_calls() {
        use agent_runtime::types::AgentEvent;

        let events = vec![
            AgentEvent::RunStart { task_name: "t".to_string() },
            AgentEvent::ToolCallStart {
                tool_name: "bash".to_string(),
                tool_call_id: "c1".to_string(),
                arguments: serde_json::json!({}),
            },
            AgentEvent::ToolCallEnd {
                tool_call_id: "c1".to_string(),
                result: "Blocked: blocked dangerous command pattern: rm -rf".to_string(),
                is_error: true,
                blocked: true,
                duration_ms: 0,
                arguments: serde_json::json!({"command": "rm -rf /"}),
            },
            AgentEvent::RunEnd { status: "blocked".to_string() },
        ];

        let trace = super::events_to_trace(&events, "2026-08-07T00:00:00Z", 1234);

        // tool_call_start + tool_call_end 合并为一个 tool_call 事件
        assert_eq!(trace.len(), 3);
        assert!(!trace.iter().any(|e| e["type"] == "tool_call_start"));

        let tool_call = &trace[1];
        assert_eq!(tool_call["type"], "tool_call");
        assert_eq!(tool_call["tool_name"], "bash");
        assert_eq!(tool_call["blocked"], true);
        assert_eq!(tool_call["duration_ms"], 0);
        assert_eq!(tool_call["result"], "Blocked: blocked dangerous command pattern: rm -rf");
        assert_eq!(tool_call["timestamp"], "2026-08-07T00:00:00Z");

        // run_end 事件携带 CLI 层写入的真实总耗时（修复 total_duration_ms 硬编码）
        let run_end = &trace[2];
        assert_eq!(run_end["type"], "run_end");
        assert_eq!(run_end["duration_ms"], 1234);
    }
}