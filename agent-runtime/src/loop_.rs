use std::sync::Arc;
use std::collections::HashMap;
use futures::StreamExt;
use crate::types::{AgentEvent, Message, Role, ToolCall, ToolResult};
use crate::provider::{LlmProvider, StreamEvent};
use crate::tool::Tool;
use crate::policy::{PermissionPolicy, Permission};
use crate::trace::TraceEmitter;

pub struct AgentConfig {
    pub system_prompt: String,
    pub max_turns: usize,
    pub tools: Vec<Arc<dyn Tool>>,
    pub policy: Arc<dyn PermissionPolicy>,
    pub provider: Arc<dyn LlmProvider>,
    pub tracer: Arc<dyn TraceEmitter>,
    pub model: String,
}

pub async fn run_agent(
    config: AgentConfig,
    task_name: &str,
    user_prompt: &str,
) -> anyhow::Result<(Vec<Message>, Vec<AgentEvent>)> {
    let mut messages: Vec<Message> = Vec::new();
    let mut events: Vec<AgentEvent> = Vec::new();

    messages.push(Message {
        role: Role::User,
        content: user_prompt.to_string(),
        tool_calls: vec![],
        tool_results: vec![],
    });

    let run_start_event = AgentEvent::RunStart { task_name: task_name.to_string() };
    config.tracer.emit(run_start_event.clone());
    events.push(run_start_event);

    let tool_defs: Vec<serde_json::Value> = config.tools.iter()
        .map(|t| t.definition())
        .map(|d| serde_json::json!({
            "name": d.name,
            "description": d.description,
            "input_schema": d.parameters,
        }))
        .collect();

    let tool_map: HashMap<String, Arc<dyn Tool>> = config.tools.iter()
        .map(|t| (t.definition().name.clone(), t.clone()))
        .collect();

    for turn in 0..config.max_turns {
        let turn_start_event = AgentEvent::TurnStart { turn: turn + 1 };
        config.tracer.emit(turn_start_event.clone());
        events.push(turn_start_event);

        let mut stream = config.provider.stream(
            &config.system_prompt,
            &messages,
            &tool_defs,
        ).await?;

        let mut assistant_text = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut stop_reason = String::new();

        while let Some(item) = stream.next().await {
            match item {
                Ok(StreamEvent::TextDelta(text)) => {
                    assistant_text.push_str(&text);
                    let event = AgentEvent::TextDelta { text };
                    config.tracer.emit(event.clone());
                    events.push(event);
                }
                Ok(StreamEvent::ToolCallStart { id, name, arguments: _ }) => {
                    let event = AgentEvent::ToolCallStart {
                        tool_name: name.clone(),
                        tool_call_id: id.clone(),
                        arguments: serde_json::json!({}),
                    };
                    config.tracer.emit(event.clone());
                    events.push(event);
                    tool_calls.push(ToolCall { id, name, arguments: serde_json::json!({}) });
                }
                Ok(StreamEvent::MessageStop { stop_reason: sr }) => {
                    stop_reason = sr;
                }
                Ok(StreamEvent::Error(e)) => {
                    let event = AgentEvent::Error { message: e };
                    config.tracer.emit(event.clone());
                    events.push(event);
                }
                _ => {}
            }
        }

        messages.push(Message {
            role: Role::Assistant,
            content: assistant_text,
            tool_calls: tool_calls.clone(),
            tool_results: vec![],
        });

        let turn_end_event = AgentEvent::TurnEnd { turn: turn + 1 };
        config.tracer.emit(turn_end_event.clone());
        events.push(turn_end_event);

        if stop_reason != "tool_use" || tool_calls.is_empty() {
            let run_end_event = AgentEvent::RunEnd { status: "success".to_string() };
            config.tracer.emit(run_end_event.clone());
            events.push(run_end_event);
            return Ok((messages, events));
        }

        let mut tool_results: Vec<ToolResult> = Vec::new();
        for tc in &tool_calls {
            let tool = tool_map.get(&tc.name);
            let (_blocked, result_content, is_error) = match tool {
                Some(t) => {
                    let permission = config.policy.check(&tc.name, &tc.arguments);
                    let blocked = matches!(permission, Permission::Deny { .. });
                    if blocked {
                        let reason = match &permission {
                            Permission::Deny { reason } => reason.clone(),
                            _ => String::new(),
                        };
                        let event = AgentEvent::ToolCallEnd {
                            tool_call_id: tc.id.clone(),
                            result: format!("Blocked: {}", reason),
                            is_error: true,
                            blocked: true,
                        };
                        config.tracer.emit(event.clone());
                        events.push(event);
                        (true, format!("Blocked: {}", reason), true)
                    } else {
                        let result = t.execute(&tc.arguments).await;
                        let event = AgentEvent::ToolCallEnd {
                            tool_call_id: tc.id.clone(),
                            result: result.content.clone(),
                            is_error: result.is_error,
                            blocked: false,
                        };
                        config.tracer.emit(event.clone());
                        events.push(event);
                        (false, result.content, result.is_error)
                    }
                }
                None => {
                    let msg = format!("Unknown tool: {}", tc.name);
                    let event = AgentEvent::ToolCallEnd {
                        tool_call_id: tc.id.clone(),
                        result: msg.clone(),
                        is_error: true,
                        blocked: false,
                    };
                    config.tracer.emit(event.clone());
                    events.push(event);
                    (false, msg, true)
                }
            };
            tool_results.push(ToolResult {
                tool_call_id: tc.id.clone(),
                content: result_content,
                is_error,
            });
        }

        messages.push(Message {
            role: Role::Tool,
            content: String::new(),
            tool_calls: vec![],
            tool_results,
        });

        let any_blocked = tool_calls.iter().any(|tc| {
            messages.iter()
                .filter(|m| m.role == Role::Tool)
                .flat_map(|m| &m.tool_results)
                .any(|r| r.tool_call_id == tc.id && r.content.starts_with("Blocked:"))
        });
        if any_blocked {
            let run_end_event = AgentEvent::RunEnd { status: "blocked".to_string() };
            config.tracer.emit(run_end_event.clone());
            events.push(run_end_event);
            return Ok((messages, events));
        }
    }

    let run_end_event = AgentEvent::RunEnd { status: "timeout".to_string() };
    config.tracer.emit(run_end_event.clone());
    events.push(run_end_event);
    Ok((messages, events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;
    use crate::policy::AllowAll;

    #[tokio::test]
    async fn test_agent_loop_simple_response() {
        let provider = Arc::new(MockProvider::new(vec!["Hello, world!".to_string()]));
        let policy: Arc<dyn PermissionPolicy> = Arc::new(AllowAll);
        let tracer = Arc::new(crate::trace::InMemoryEmitter::new());

        let config = AgentConfig {
            system_prompt: "You are helpful.".to_string(),
            max_turns: 5,
            tools: vec![],
            policy,
            provider,
            tracer: tracer.clone(),
            model: "test-model".to_string(),
        };

        let (messages, events) = run_agent(config, "test", "Say hello").await.unwrap();
        assert!(!messages.is_empty());
        assert!(events.iter().any(|e| matches!(e, AgentEvent::RunEnd { status } if status == "success")));
    }
}
