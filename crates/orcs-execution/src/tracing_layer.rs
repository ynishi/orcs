//! Custom tracing layer for streaming orchestrator events to Tauri frontend
//!
//! This module provides a tracing layer that captures orchestration events
//! and forwards them to the Tauri frontend via tokio channels.

use orcs_core::task::Task;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

/// Event data sent to the frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct OrchestratorEvent {
    /// Event target (e.g., "llm_toolkit::orchestrator::parallel_orchestrator")
    pub target: String,
    /// Log level (INFO, DEBUG, WARN, ERROR)
    pub level: String,
    /// Human-readable message
    pub message: String,
    /// Structured fields from the event
    pub fields: HashMap<String, Value>,
    /// Span context fields (e.g., wave_number, step_id)
    pub span: HashMap<String, Value>,
    /// Timestamp
    pub timestamp: String,
    /// Event type marker for manual events (e.g., "task_lifecycle", "orchestrator_trace")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type: Option<String>,
}

/// A custom tracing layer that sends orchestrator events to a channel
pub struct OrchestratorEventLayer {
    sender: mpsc::UnboundedSender<OrchestratorEvent>,
}

impl OrchestratorEventLayer {
    /// Create a new layer with the given channel sender
    pub fn new(sender: mpsc::UnboundedSender<OrchestratorEvent>) -> Self {
        Self { sender }
    }
}

impl<S> Layer<S> for OrchestratorEventLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut fields = HashMap::new();
        let mut visitor = FieldVisitor(&mut fields);
        event.record(&mut visitor);

        // Extract span context
        let mut span_fields = HashMap::new();

        // Extract span name for context (e.g., "wave")
        if let Some(span_id) = ctx.current_span().id() {
            if let Some(span) = ctx.span(span_id) {
                let metadata = span.metadata();
                span_fields.insert("span_name".to_string(), serde_json::json!(metadata.name()));
            }
        }

        // Extract wave_number from message if present
        // Message format: "Executing wave 2 with 3 steps"
        let wave_number = {
            let message_str = fields.get("message").and_then(|v| v.as_str()).unwrap_or("");

            if message_str.contains("Executing wave ") {
                message_str
                    .split("Executing wave ")
                    .nth(1)
                    .and_then(|s| s.split(" with").next())
                    .and_then(|num_str| num_str.parse::<u64>().ok())
            } else {
                None
            }
        };

        if let Some(wave_num) = wave_number {
            fields.insert("wave_number".to_string(), serde_json::json!(wave_num));
        }

        let orchestrator_event = OrchestratorEvent {
            target: event.metadata().target().to_string(),
            level: event.metadata().level().to_string(),
            message: fields
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            fields,
            span: span_fields,
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: None, // Auto-generated tracing events have no type marker
        };

        // Non-blocking send - if the receiver is dropped or full, we just skip
        let _ = self.sender.send(orchestrator_event);
    }
}

/// Field visitor that extracts tracing event fields into a HashMap
struct FieldVisitor<'a>(&'a mut HashMap<String, Value>);

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), serde_json::json!(value));
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::json!(format!("{:?}", value)),
        );
    }
}

// ============================================================================
// Event Builder - Type-safe helper for creating task-related events
// ============================================================================

/// Builder for creating OrchestratorEvent instances with type safety.
///
/// This builder ensures consistent field naming and prevents typos/missing fields
/// in task-related events sent to the frontend.
pub struct OrchestratorEventBuilder {
    target: String,
    level: String,
    message: String,
    fields: HashMap<String, Value>,
    event_type: Option<String>,
}

impl OrchestratorEventBuilder {
    /// Creates a new builder with INFO level from a Task object.
    ///
    /// This automatically populates all task fields into the event.
    pub fn info_from_task(message: impl Into<String>, task: &Task) -> Self {
        let mut fields = HashMap::new();

        // Core identifiers
        fields.insert("task_id".to_string(), serde_json::json!(&task.id));
        fields.insert(
            "session_id".to_string(),
            serde_json::json!(&task.session_id),
        );

        // Task metadata
        fields.insert("title".to_string(), serde_json::json!(&task.title));
        fields.insert(
            "description".to_string(),
            serde_json::json!(&task.description),
        );
        fields.insert(
            "status".to_string(),
            serde_json::json!(task.status.as_str()),
        );

        // Timestamps
        fields.insert(
            "created_at".to_string(),
            serde_json::json!(&task.created_at),
        );
        fields.insert(
            "updated_at".to_string(),
            serde_json::json!(&task.updated_at),
        );
        if let Some(ref completed_at) = task.completed_at {
            fields.insert("completed_at".to_string(), serde_json::json!(completed_at));
        }

        // Execution metrics
        fields.insert(
            "steps_executed".to_string(),
            serde_json::json!(task.steps_executed),
        );
        fields.insert(
            "steps_skipped".to_string(),
            serde_json::json!(task.steps_skipped),
        );
        fields.insert(
            "context_keys".to_string(),
            serde_json::json!(task.context_keys),
        );

        // Optional fields
        if let Some(ref error) = task.error {
            fields.insert("error".to_string(), serde_json::json!(error));
        }
        if let Some(ref result) = task.result {
            fields.insert("result".to_string(), serde_json::json!(result));
        }

        eprintln!("[EventBuilder] info_from_task called:");
        eprintln!("  task_id: {}", &task.id);
        eprintln!("  status: {}", task.status.as_str());
        eprintln!("  fields count: {}", fields.len());
        eprintln!("  fields keys: {:?}", fields.keys().collect::<Vec<_>>());
        use std::io::Write;
        let _ = std::io::stderr().flush();

        Self {
            target: "orcs_execution::task_executor".to_string(),
            level: "INFO".to_string(),
            message: message.into(),
            fields,
            event_type: Some("task_lifecycle".to_string()),
        }
    }

    /// Creates a new builder with ERROR level from a Task object.
    ///
    /// This automatically populates all task fields into the event.
    pub fn error_from_task(message: impl Into<String>, task: &Task) -> Self {
        let mut builder = Self::info_from_task(message, task);
        builder.level = "ERROR".to_string();
        builder
    }

    /// Creates a new builder with INFO level and default target.
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            target: "orcs_execution::task_executor".to_string(),
            level: "INFO".to_string(),
            message: message.into(),
            fields: HashMap::new(),
            event_type: Some("task_lifecycle".to_string()),
        }
    }

    /// Creates a new builder with ERROR level and default target.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            target: "orcs_execution::task_executor".to_string(),
            level: "ERROR".to_string(),
            message: message.into(),
            fields: HashMap::new(),
            event_type: Some("task_lifecycle".to_string()),
        }
    }

    /// Sets the event target (overrides default).
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// Adds a custom field (for advanced use cases).
    pub fn field(mut self, key: impl Into<String>, value: impl serde::Serialize) -> Self {
        self.fields.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    /// Builds the final OrchestratorEvent.
    pub fn build(self) -> OrchestratorEvent {
        let event = OrchestratorEvent {
            target: self.target,
            level: self.level,
            message: self.message,
            fields: self.fields,
            span: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            event_type: self.event_type,
        };

        use std::io::Write;
        let _ = std::io::stderr().flush();

        event
    }
}
