//! Distributed tracing for CNWS
//! Implements OpenTelemetry-compatible tracing

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{field, span, Level, Span};
use tracing_subscriber::Layer;

/// Trace context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Trace flags
    pub trace_flags: u8,
    /// Trace state
    pub trace_state: Option<String>,
}

impl TraceContext {
    /// Create a new trace context
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            trace_flags: 0x01, // Sampled
            trace_state: None,
        }
    }

    /// Create root context
    pub fn root() -> Self {
        let trace_id = generate_trace_id();
        let span_id = generate_span_id();
        Self::new(trace_id, span_id)
    }

    /// Create child context
    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
        }
    }
}

/// Span data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanData {
    /// Trace context
    pub context: TraceContext,
    /// Operation name
    pub name: String,
    /// Span kind
    pub kind: SpanKind,
    /// Start timestamp (microseconds)
    pub start_time_us: u64,
    /// End timestamp (microseconds)
    pub end_time_us: Option<u64>,
    /// Duration (microseconds)
    pub duration_us: Option<u64>,
    /// Status
    pub status: SpanStatus,
    /// Attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Events
    pub events: Vec<SpanEvent>,
}

impl SpanData {
    /// Create a new span
    pub fn new(name: impl Into<String>, context: TraceContext) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        Self {
            context,
            name: name.into(),
            kind: SpanKind::Internal,
            start_time_us: now,
            end_time_us: None,
            duration_us: None,
            status: SpanStatus::Ok,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// End the span
    pub fn end(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        self.end_time_us = Some(now);
        self.duration_us = Some(now - self.start_time_us);
    }

    /// Add attribute
    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Add event
    pub fn add_event(&mut self, event: SpanEvent) {
        self.events.push(event);
    }
}

/// Span kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanKind {
    /// Internal span
    Internal,
    /// Server span
    Server,
    /// Client span
    Client,
    /// Producer span
    Producer,
    /// Consumer span
    Consumer,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Ok
    Ok,
    /// Error
    Error,
    /// Unset
    Unset,
}

/// Span event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// Timestamp (microseconds)
    pub timestamp_us: u64,
    /// Attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

impl SpanEvent {
    /// Create a new span event
    pub fn new(name: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        Self {
            name: name.into(),
            timestamp_us: now,
            attributes: HashMap::new(),
        }
    }
}

/// CNWS tracer
pub struct CnwsTracer {
    spans: Arc<parking_lot::Mutex<Vec<SpanData>>>,
}

impl CnwsTracer {
    /// Create a new tracer
    pub fn new() -> Self {
        Self {
            spans: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }

    /// Start a new span
    pub fn start_span(&self, name: impl Into<String>, context: TraceContext) -> SpanGuard {
        let span_data = SpanData::new(name, context);
        let id = span_data.context.span_id.clone();

        self.spans.lock().push(span_data.clone());

        SpanGuard {
            tracer: self,
            span_data,
        }
    }

    /// Start a child span
    pub fn start_child_span(&self, name: impl Into<String>, parent: &TraceContext) -> SpanGuard {
        let child_context = parent.child();
        self.start_span(name, child_context)
    }

    /// Get all spans
    pub fn spans(&self) -> Vec<SpanData> {
        self.spans.lock().clone()
    }

    /// Clear spans
    pub fn clear(&self) {
        self.spans.lock().clear();
    }

    /// Get span count
    pub fn span_count(&self) -> usize {
        self.spans.lock().len()
    }
}

impl Default for CnwsTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Span guard - automatically ends span when dropped
pub struct SpanGuard<'a> {
    tracer: &'a CnwsTracer,
    span_data: SpanData,
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        let mut span_data = SpanData {
            context: self.span_data.context.clone(),
            name: self.span_data.name.clone(),
            kind: self.span_data.kind,
            start_time_us: self.span_data.start_time_us,
            end_time_us: None,
            duration_us: None,
            status: self.span_data.status,
            attributes: self.span_data.attributes.clone(),
            events: self.span_data.events.clone(),
        };
        span_data.end();

        // Update span in list
        let mut spans = self.tracer.spans.lock();
        if let Some(span) = spans.iter_mut().find(|s| s.context.span_id == span_data.context.span_id) {
            *span = span_data;
        }
    }
}

impl<'a> SpanGuard<'a> {
    /// Add attribute
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.span_data.add_attribute(key, value);
        self
    }

    /// Add event
    pub fn event(mut self, event: SpanEvent) -> Self {
        self.span_data.add_event(event);
        self
    }

    /// Set status
    pub fn status(mut self, status: SpanStatus) -> Self {
        self.span_data.status = status;
        self
    }
}

/// Generate a random trace ID (16 bytes = 32 hex chars)
fn generate_trace_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", now)
}

/// Generate a random span ID (8 bytes = 16 hex chars)
fn generate_span_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:016x}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_root() {
        let ctx = TraceContext::root();
        assert_eq!(ctx.trace_id.len(), 32);
        assert_eq!(ctx.span_id.len(), 16);
        assert!(ctx.parent_span_id.is_none());
    }

    #[test]
    fn test_trace_context_child() {
        let parent = TraceContext::root();
        let child = parent.child();
        assert_eq!(child.trace_id, parent.trace_id);
        assert_eq!(child.parent_span_id, Some(parent.span_id));
    }

    #[test]
    fn test_tracer() {
        let tracer = CnwsTracer::new();
        let ctx = TraceContext::root();
        let _guard = tracer.start_span("test_span", ctx);
        assert_eq!(tracer.span_count(), 1);
    }
}
