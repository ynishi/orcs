//! Custom tracing layer for streaming orchestrator events to Tauri frontend
//!
//! This module provides a tracing layer that captures orchestration events
//! and forwards them to the Tauri frontend via tokio channels.

use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

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
        if let Some(span_id) = ctx.current_span().id() {
            if let Some(span) = ctx.span(span_id) {
                let extensions = span.extensions();
                if let Some(stored) = extensions.get::<HashMap<String, Value>>() {
                    span_fields = stored.clone();
                }
            }
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
