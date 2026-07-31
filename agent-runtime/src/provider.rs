use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use crate::types::Message;

pub type StreamItem = Result<StreamEvent>;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    TextDelta(String),
    ThinkingDelta(String),
    ToolCallStart { id: String, name: String, arguments: String },
    ToolCallFull { id: String, name: String, arguments: serde_json::Value },
    MessageStop { stop_reason: String },
    Error(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>>;
}

/// Accumulator for a tool call that is being streamed by a provider.
/// The `arguments` field grows as `input_json_delta` / `function.arguments`
/// fragments arrive, until the tool call completes.
#[derive(Debug)]
struct PendingToolCall {
    id: String,
    name: String,
    arguments: String,
    started: bool,
}

/// Parse an accumulated JSON string into a `Value`. If the accumulated
/// fragments do not form valid JSON, fall back to a raw string value so no
/// data is silently dropped.
fn parse_arguments(acc: &str) -> serde_json::Value {
    serde_json::from_str(acc).unwrap_or_else(|_| serde_json::Value::String(acc.to_string()))
}

/// Emit `ToolCallFull` for every pending tool call in order, then clear them.
fn flush_tool_calls(pending: &mut HashMap<u64, PendingToolCall>, out: &mut Vec<StreamItem>) {
    let mut keys: Vec<u64> = pending.keys().cloned().collect();
    keys.sort();
    for k in keys {
        if let Some(tc) = pending.remove(&k) {
            let arguments = parse_arguments(&tc.arguments);
            out.push(Ok(StreamEvent::ToolCallFull {
                id: tc.id,
                name: tc.name,
                arguments,
            }));
        }
    }
}

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;
        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-5@20250929".to_string());
        Ok(Self::new(api_key, model))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        let mut anthropic_messages: Vec<serde_json::Value> = Vec::new();
        for msg in messages {
            let role = match msg.role {
                crate::types::Role::User => "user",
                crate::types::Role::Assistant => "assistant",
                _ => continue,
            };
            let mut content: Vec<serde_json::Value> = Vec::new();
            if !msg.content.is_empty() {
                content.push(serde_json::json!({"type": "text", "text": msg.content}));
            }
            for tc in &msg.tool_calls {
                content.push(serde_json::json!({
                    "type": "tool_use", "id": tc.id, "name": tc.name, "input": tc.arguments
                }));
            }
            for tr in &msg.tool_results {
                content.push(serde_json::json!({
                    "type": "tool_result", "tool_use_id": tr.tool_call_id,
                    "content": tr.content, "is_error": tr.is_error
                }));
            }
            anthropic_messages.push(serde_json::json!({"role": role, "content": content}));
        }

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": anthropic_messages,
            "stream": true
        });
        if !system_prompt.is_empty() {
            body["system"] = serde_json::json!(system_prompt);
        }
        if !tools.is_empty() {
            body["tools"] = serde_json::json!(tools);
        }

        let response = self.client.post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error {}: {}", status, text));
        }

        use futures::StreamExt;
        let mut pending: HashMap<u64, PendingToolCall> = HashMap::new();
        let stream = response.bytes_stream().flat_map(move |chunk| {
            let mut out: Vec<StreamItem> = Vec::new();
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    out.push(Ok(StreamEvent::Error(e.to_string())));
                    return futures::stream::iter(out);
                }
            };
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { continue; }
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                        let event_type = event["type"].as_str().unwrap_or("");
                        match event_type {
                            "content_block_start" => {
                                let block = &event["content_block"];
                                if block["type"] == "tool_use" {
                                    let index = event["index"].as_u64().unwrap_or(0);
                                    let id = block["id"].as_str().unwrap_or("").to_string();
                                    let name = block["name"].as_str().unwrap_or("").to_string();
                                    pending.insert(index, PendingToolCall {
                                        id: id.clone(),
                                        name: name.clone(),
                                        arguments: String::new(),
                                        started: true,
                                    });
                                    out.push(Ok(StreamEvent::ToolCallStart {
                                        id,
                                        name,
                                        arguments: String::new(),
                                    }));
                                }
                            }
                            "content_block_delta" => {
                                let delta = &event["delta"];
                                let delta_type = delta["type"].as_str().unwrap_or("");
                                match delta_type {
                                    "text_delta" => {
                                        out.push(Ok(StreamEvent::TextDelta(
                                            delta["text"].as_str().unwrap_or("").to_string()
                                        )));
                                    }
                                    "thinking_delta" => {
                                        out.push(Ok(StreamEvent::ThinkingDelta(
                                            delta["thinking"].as_str().unwrap_or("").to_string()
                                        )));
                                    }
                                    "input_json_delta" => {
                                        let index = event["index"].as_u64().unwrap_or(0);
                                        if let Some(tc) = pending.get_mut(&index) {
                                            tc.arguments.push_str(
                                                delta["partial_json"].as_str().unwrap_or("")
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            "content_block_stop" => {
                                let index = event["index"].as_u64().unwrap_or(0);
                                if let Some(tc) = pending.remove(&index) {
                                    let arguments = parse_arguments(&tc.arguments);
                                    out.push(Ok(StreamEvent::ToolCallFull {
                                        id: tc.id,
                                        name: tc.name,
                                        arguments,
                                    }));
                                }
                            }
                            "message_delta" => {
                                if let Some(stop) = event["delta"]["stop_reason"].as_str() {
                                    out.push(Ok(StreamEvent::MessageStop {
                                        stop_reason: stop.to_string()
                                    }));
                                }
                            }
                            "message_stop" => {
                                flush_tool_calls(&mut pending, &mut out);
                                out.push(Ok(StreamEvent::MessageStop {
                                    stop_reason: "end_turn".to_string()
                                }));
                            }
                            _ => {}
                        }
                    }
                }
            }
            futures::stream::iter(out)
        });
        Ok(Box::pin(stream))
    }
}

/// Internal OpenAI-compatible chat completions provider. Both DeepSeek and
/// OpenAI expose this format, so they share this single implementation to
/// avoid duplicating message serialization and SSE parsing.
struct OpenAICompatibleProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAICompatibleProvider {
    fn new(api_key: String, model: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url,
        }
    }

    fn from_env(api_key_var: &str, model_var: &str, default_model: &str, base_url: &str) -> Result<Self> {
        let api_key = std::env::var(api_key_var)
            .map_err(|_| anyhow::anyhow!("{} not set", api_key_var))?;
        let model = std::env::var(model_var).unwrap_or_else(|_| default_model.to_string());
        Ok(Self::new(api_key, model, base_url.to_string()))
    }

    async fn stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        let mut openai_messages: Vec<serde_json::Value> = Vec::new();

        if !system_prompt.is_empty() {
            openai_messages.push(serde_json::json!({
                "role": "system",
                "content": system_prompt
            }));
        }

        for msg in messages {
            match msg.role {
                crate::types::Role::User => {
                    openai_messages.push(serde_json::json!({
                        "role": "user", "content": msg.content
                    }));
                }
                crate::types::Role::Assistant => {
                    let mut m = serde_json::json!({
                        "role": "assistant", "content": msg.content
                    });
                    if !msg.tool_calls.is_empty() {
                        let tcs: Vec<serde_json::Value> = msg.tool_calls.iter().map(|tc| {
                            serde_json::json!({
                                "id": tc.id, "type": "function",
                                "function": { "name": tc.name, "arguments": tc.arguments.to_string() }
                            })
                        }).collect();
                        m["tool_calls"] = serde_json::json!(tcs);
                    }
                    openai_messages.push(m);
                }
                crate::types::Role::Tool => {
                    for tr in &msg.tool_results {
                        openai_messages.push(serde_json::json!({
                            "role": "tool", "tool_call_id": tr.tool_call_id, "content": tr.content
                        }));
                    }
                }
                _ => continue,
            }
        }

        let openai_tools: Vec<serde_json::Value> = tools.iter().map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t["name"],
                    "description": t["description"],
                    "parameters": t["input_schema"]
                }
            })
        }).collect();

        let mut body = serde_json::json!({
            "model": self.model,
            "messages": openai_messages,
            "stream": true,
            "max_tokens": 4096
        });
        if !openai_tools.is_empty() {
            body["tools"] = serde_json::json!(openai_tools);
        }

        let response = self.client.post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI-compatible API error {}: {}", status, text));
        }

        use futures::StreamExt;
        let mut pending: HashMap<u64, PendingToolCall> = HashMap::new();
        let stream = response.bytes_stream().flat_map(move |chunk| {
            let mut out: Vec<StreamItem> = Vec::new();
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    out.push(Ok(StreamEvent::Error(e.to_string())));
                    return futures::stream::iter(out);
                }
            };
            let text = String::from_utf8_lossy(&chunk);
            let mut stopped = false;

            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        stopped = true;
                        break;
                    }
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(choices) = event["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                // Accumulate any tool call fragments first, so a chunk
                                // that carries both the final fragment and finish_reason
                                // still captures the complete arguments.
                                if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
                                    for tc in tool_calls {
                                        let index = tc["index"].as_u64().unwrap_or(0);
                                        let id = tc["id"].as_str().unwrap_or("").to_string();
                                        let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                                        let args = tc["function"]["arguments"].as_str().unwrap_or("");

                                        let entry = pending.entry(index).or_insert_with(|| PendingToolCall {
                                            id: String::new(),
                                            name: String::new(),
                                            arguments: String::new(),
                                            started: false,
                                        });
                                        if !id.is_empty() {
                                            entry.id = id;
                                        }
                                        if !name.is_empty() {
                                            entry.name = name;
                                        }
                                        if !entry.started && (!entry.name.is_empty() || !entry.id.is_empty()) {
                                            entry.started = true;
                                            out.push(Ok(StreamEvent::ToolCallStart {
                                                id: entry.id.clone(),
                                                name: entry.name.clone(),
                                                arguments: String::new(),
                                            }));
                                        }
                                        if !args.is_empty() {
                                            entry.arguments.push_str(args);
                                        }
                                    }
                                }

                                if let Some(content) = choice["delta"]["content"].as_str() {
                                    if !content.is_empty() {
                                        out.push(Ok(StreamEvent::TextDelta(content.to_string())));
                                    }
                                }

                                if let Some(reasoning) = choice["delta"]["reasoning_content"].as_str() {
                                    if !reasoning.is_empty() {
                                        out.push(Ok(StreamEvent::ThinkingDelta(reasoning.to_string())));
                                    }
                                }

                                if let Some(finish) = choice["finish_reason"].as_str() {
                                    if !finish.is_empty() && finish != "null" {
                                        let mapped = match finish {
                                            "tool_calls" => "tool_use",
                                            _ => finish,
                                        };
                                        // Emit complete tool calls before the stop marker.
                                        flush_tool_calls(&mut pending, &mut out);
                                        out.push(Ok(StreamEvent::MessageStop {
                                            stop_reason: mapped.to_string()
                                        }));
                                        stopped = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if stopped {
                flush_tool_calls(&mut pending, &mut out);
                if !out.iter().any(|it| matches!(it, Ok(StreamEvent::MessageStop { .. }))) {
                    out.push(Ok(StreamEvent::MessageStop {
                        stop_reason: "end_turn".to_string()
                    }));
                }
            }

            futures::stream::iter(out)
        });
        Ok(Box::pin(stream))
    }
}

pub struct DeepseekProvider {
    inner: OpenAICompatibleProvider,
}

impl DeepseekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            inner: OpenAICompatibleProvider::new(
                api_key,
                model,
                "https://api.deepseek.com/v1/chat/completions".to_string(),
            ),
        }
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            inner: OpenAICompatibleProvider::from_env(
                "DEEPSEEK_API_KEY",
                "DEEPSEEK_MODEL",
                "deepseek-chat",
                "https://api.deepseek.com/v1/chat/completions",
            )?,
        })
    }
}

#[async_trait]
impl LlmProvider for DeepseekProvider {
    async fn stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        self.inner.stream(system_prompt, messages, tools).await
    }
}

pub struct OpenAIProvider {
    inner: OpenAICompatibleProvider,
}

impl OpenAIProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            inner: OpenAICompatibleProvider::new(
                api_key,
                model,
                "https://api.openai.com/v1/chat/completions".to_string(),
            ),
        }
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            inner: OpenAICompatibleProvider::from_env(
                "OPENAI_API_KEY",
                "OPENAI_MODEL",
                "gpt-4o",
                "https://api.openai.com/v1/chat/completions",
            )?,
        })
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn stream(
        &self,
        system_prompt: &str,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        self.inner.stream(system_prompt, messages, tools).await
    }
}

#[cfg(test)]
pub struct MockProvider {
    /// One scripted stream of events per `stream()` call.
    pub responses: Vec<Vec<StreamEvent>>,
    pub call_count: std::sync::Mutex<usize>,
}

#[cfg(test)]
impl MockProvider {
    /// Build a provider that emits the given text response followed by a stop.
    pub fn new_text(responses: Vec<String>) -> Self {
        let mut events: Vec<StreamEvent> =
            responses.into_iter().map(StreamEvent::TextDelta).collect();
        events.push(StreamEvent::MessageStop { stop_reason: "end_turn".to_string() });
        Self { responses: vec![events], call_count: std::sync::Mutex::new(0) }
    }

    /// Build a provider from a scripted sequence of stream events for a single call.
    pub fn new(responses: Vec<StreamEvent>) -> Self {
        Self { responses: vec![responses], call_count: std::sync::Mutex::new(0) }
    }
}

#[cfg(test)]
#[async_trait]
impl LlmProvider for MockProvider {
    async fn stream(
        &self, _system_prompt: &str, _messages: &[Message], _tools: &[serde_json::Value],
    ) -> Result<Pin<Box<dyn Stream<Item = StreamItem> + Send>>> {
        let mut count = self.call_count.lock().unwrap();
        let idx = *count;
        *count += 1;
        let events: Vec<StreamEvent> = match self.responses.get(idx) {
            Some(ev) => ev.clone(),
            None => Vec::new(),
        };
        let items: Vec<StreamItem> = events.into_iter().map(Ok).collect();
        Ok(Box::pin(futures::stream::iter(items)))
    }
}
