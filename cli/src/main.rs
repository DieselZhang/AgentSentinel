use clap::{Parser, Subcommand};
use agent_runtime::loop_::{run_agent, AgentConfig};
use agent_runtime::provider::{AnthropicProvider, DeepseekProvider, LlmProvider};
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

fn event_type_str(e: &agent_runtime::types::AgentEvent) -> &str {
    match e {
        agent_runtime::types::AgentEvent::TextDelta{..} => "text_delta",
        agent_runtime::types::AgentEvent::ThinkingDelta{..} => "thinking",
        agent_runtime::types::AgentEvent::ToolCallStart{..} => "tool_call_start",
        agent_runtime::types::AgentEvent::ToolCallEnd{..} => "tool_call_end",
        agent_runtime::types::AgentEvent::TurnStart{..} => "turn_start",
        agent_runtime::types::AgentEvent::TurnEnd{..} => "turn_end",
        agent_runtime::types::AgentEvent::RunStart{..} => "run_start",
        agent_runtime::types::AgentEvent::RunEnd{..} => "run_end",
        agent_runtime::types::AgentEvent::Error{..} => "error",
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { task, prompt, system, max_turns, provider } => {
            let provider: Arc<dyn LlmProvider> = match provider.as_str() {
                "deepseek" | "ds" => Arc::new(DeepseekProvider::from_env()?),
                _ => Arc::new(AnthropicProvider::from_env()?),
            };
            let policy: Arc<dyn PermissionPolicy> = Arc::new(DenyDangerous::default());
            let tracer = Arc::new(InMemoryEmitter::new());
            let tools: Vec<Arc<dyn Tool>> = vec![Arc::new(ReadFile), Arc::new(WriteFile), Arc::new(Bash)];
            let config = AgentConfig { system_prompt: system, max_turns, tools, policy, provider, tracer: tracer.clone(), model: std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_|"claude-sonnet-4-5@20250929".into()) };
            println!("Running: {}", task);
            let start = std::time::Instant::now();
            match run_agent(config, &task, &prompt).await {
                Ok((messages, events)) => {
                    let status = events.iter().rev().find_map(|e| match e { agent_runtime::types::AgentEvent::RunEnd{status} => Some(status.clone()), _ => None }).unwrap_or("unknown".into());
                    let turns = messages.iter().filter(|m| m.role == agent_runtime::types::Role::Assistant).count();
                    let now = chrono::Utc::now().to_rfc3339();
                    let trace_events: Vec<serde_json::Value> = events.iter().map(|e| serde_json::json!({"timestamp":now,"event_type":event_type_str(e),"data":e})).collect();
                    let fname = format!("traces/{}_{}.json", task, chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                    std::fs::create_dir_all("traces")?;
                    std::fs::write(&fname, &serde_json::to_string_pretty(&trace_events)?)?;
                    println!("Status: {}, Turns: {}, Duration: {}ms", status, turns, start.elapsed().as_millis());
                    println!("Trace: {}", fname);
                }
                Err(e) => eprintln!("Failed: {}", e),
            }
        }
        Commands::Upload { file, server } => {
            let content = std::fs::read_to_string(&file)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            let task = file.replace("traces/","").replace(".json","");
            let c = reqwest::Client::new();
            let r = c.post(format!("{}/api/runs",server)).json(&serde_json::json!({"task_name":task,"model":"unknown","system_prompt":"","max_turns":10,"events_json":serde_json::to_string(&json).unwrap_or_default(),"status":"success","total_turns":1,"total_tokens":0,"total_duration_ms":0})).send().await?;
            if r.status().is_success() { let v: serde_json::Value = r.json().await?; println!("Uploaded! ID: {}", v["run_id"].as_str().unwrap_or("")); } else { eprintln!("Upload failed: {}", r.status()); }
        }
    }
    Ok(())
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
}