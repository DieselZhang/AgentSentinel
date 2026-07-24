use std::sync::Arc;
use tokio::sync::Mutex;
use crate::types::AgentEvent;

pub trait TraceEmitter: Send + Sync {
    fn emit(&self, event: AgentEvent);
}

pub struct InMemoryEmitter {
    pub events: Arc<Mutex<Vec<AgentEvent>>>,
}

impl InMemoryEmitter {
    pub fn new() -> Self {
        Self { events: Arc::new(Mutex::new(Vec::new())) }
    }
}

impl TraceEmitter for InMemoryEmitter {
    fn emit(&self, event: AgentEvent) {
        let events = self.events.clone();
        tokio::spawn(async move {
            events.lock().await.push(event);
        });
    }
}

pub struct NoopEmitter;

impl TraceEmitter for NoopEmitter {
    fn emit(&self, _event: AgentEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_emitter() {
        let emitter = InMemoryEmitter::new();
        emitter.emit(AgentEvent::RunStart { task_name: "test".to_string() });
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let events = emitter.events.lock().await;
        assert_eq!(events.len(), 1);
    }
}
