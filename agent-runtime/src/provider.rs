use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
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
        let stream = response.bytes_stream().map(|chunk| {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => return Ok(StreamEvent::Error(e.to_string())),
            };
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { continue; }
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                        let event_type = event["type"].as_str().unwrap_or("");
                        match event_type {
                            "content_block_delta" => {
                                let delta = &event["delta"];
                                if delta["type"] == "text_delta" {
                                    return Ok(StreamEvent::TextDelta(
                                        delta["text"].as_str().unwrap_or("").to_string()
                                    ));
                                }
                            }
                            "content_block_start" => {
                                let block = &event["content_block"];
                                if block["type"] == "tool_use" {
                                    return Ok(StreamEvent::ToolCallStart {
                                        id: block["id"].as_str().unwrap_or("").to_string(),
                                        name: block["name"].as_str().unwrap_or("").to_string(),
                                        arguments: String::new(),
                                    });
                                }
                            }
                            "message_delta" => {
                                if let Some(stop) = event["delta"]["stop_reason"].as_str() {
                                    return Ok(StreamEvent::MessageStop { stop_reason: stop.to_string() });
                                }
                            }
                            "message_stop" => {
                                return Ok(StreamEvent::MessageStop { stop_reason: "end_turn".to_string() });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(StreamEvent::TextDelta(String::new()))
        });
        Ok(Box::pin(stream))
    }
}

pub struct DeepseekProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl DeepseekProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DEEPSEEK_API_KEY")
            .map_err(|_| anyhow::anyhow!("DEEPSEEK_API_KEY not set"))?;
        let model = std::env::var("DEEPSEEK_MODEL")
            .unwrap_or_else(|_| "deepseek-chat".to_string());
        Ok(Self::new(api_key, model))
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
            return Err(anyhow::anyhow!("DeepSeek API error {}: {}", status, text));
        }

        use futures::StreamExt;
        let stream = response.bytes_stream().map(|chunk| {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => return Ok(StreamEvent::Error(e.to_string())),
            };
            let text = String::from_utf8_lossy(&chunk);
            let mut result: Option<StreamItem> = None;

            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        result = Some(Ok(StreamEvent::MessageStop { stop_reason: "end_turn".to_string() }));
                        continue;
                    }
                    if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(choices) = event["choices"].as_array() {
                            if let Some(choice) = choices.first() {
                                if let Some(finish) = choice["finish_reason"].as_str() {
                                    if !finish.is_empty() && finish != "null" {
                                        let mapped = match finish {
                                            "tool_calls" => "tool_use",
                                            _ => finish,
                                        };
                                        result = Some(Ok(StreamEvent::MessageStop {
                                            stop_reason: mapped.to_string()
                                        }));
                                    }
                                }

                                if let Some(content) = choice["delta"]["content"].as_str() {
                                    if !content.is_empty() {
                                        result = Some(Ok(StreamEvent::TextDelta(content.to_string())));
                                    }
                                }

                                if let Some(reasoning) = choice["delta"]["reasoning_content"].as_str() {
                                    if !reasoning.is_empty() {
                                        result = Some(Ok(StreamEvent::ThinkingDelta(reasoning.to_string())));
                                    }
                                }

                                if let Some(tool_calls) = choice["delta"]["tool_calls"].as_array() {
                                    if let Some(tc) = tool_calls.first() {
                                        let id = tc["id"].as_str().unwrap_or("");
                                        let name = tc["function"]["name"].as_str().unwrap_or("");
                                        let args = tc["function"]["arguments"].as_str().unwrap_or("");
                                        if !name.is_empty() {
                                            result = Some(Ok(StreamEvent::ToolCallStart {
                                                id: id.to_string(),
                                                name: name.to_string(),
                                                arguments: args.to_string(),
                                            }));
                                        } else if !args.is_empty() {
                                            result = Some(Ok(StreamEvent::ToolCallStart {
                                                id: id.to_string(),
                                                name: String::new(),
                                                arguments: args.to_string(),
                                            }));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            result.unwrap_or(Ok(StreamEvent::TextDelta(String::new())))
        });
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
pub struct MockProvider {
    pub responses: Vec<String>,
    pub call_count: std::sync::Mutex<usize>,
}

#[cfg(test)]
impl MockProvider {
    pub fn new(responses: Vec<String>) -> Self {
        Self { responses, call_count: std::sync::Mutex::new(0) }
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
        let response = self.responses.get(idx).cloned().unwrap_or_default();
        let items: Vec<StreamItem> = vec![Ok(StreamEvent::TextDelta(response)), Ok(StreamEvent::MessageStop { stop_reason: "end_turn".to_string() })];
        Ok(Box::pin(futures::stream::iter(items)))
    }
}
