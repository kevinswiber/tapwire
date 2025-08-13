# Task C.4: Add Telemetry/Metrics

## Overview

Implement comprehensive telemetry and metrics collection for Shadowcat using OpenTelemetry for distributed tracing and Prometheus for metrics exposition. This enables production monitoring, debugging, and performance analysis.

**Duration**: 4 hours  
**Dependencies**: Phase B (Library Readiness)  
**Status**: Not Started

## Objectives

1. Integrate OpenTelemetry for distributed tracing
2. Add Prometheus metrics endpoint for monitoring
3. Instrument key proxy operations with spans and metrics
4. Provide configurable sampling and filtering
5. Ensure minimal performance overhead when disabled

## Design Considerations

### Telemetry Architecture

```
┌─────────────────────────────────────────┐
│           Shadowcat Process             │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │     OpenTelemetry SDK           │   │
│  │  ┌──────────┐  ┌─────────────┐  │   │
│  │  │  Tracer  │  │   Metrics   │  │   │
│  │  └──────────┘  └─────────────┘  │   │
│  └──────────┬───────────┬──────────┘   │
│             │           │              │
│      OTLP Exporter  Prometheus         │
│             │         Exporter         │
└─────────────┼───────────┼──────────────┘
              │           │
              ▼           ▼
        OTLP Collector  Prometheus
         (Jaeger,      Scraper
          Tempo, etc)
```

### Key Metrics to Track

#### Proxy Metrics
- `shadowcat_proxy_requests_total` - Total requests by transport, direction
- `shadowcat_proxy_request_duration_seconds` - Request latency histogram
- `shadowcat_proxy_active_connections` - Current active connections
- `shadowcat_proxy_errors_total` - Errors by type and transport
- `shadowcat_proxy_bytes_transferred` - Data transfer by direction

#### Session Metrics
- `shadowcat_sessions_active` - Current active sessions
- `shadowcat_sessions_created_total` - Total sessions created
- `shadowcat_session_duration_seconds` - Session lifetime histogram
- `shadowcat_session_messages_total` - Messages per session

#### Transport Metrics
- `shadowcat_transport_connect_duration_seconds` - Connection time
- `shadowcat_transport_errors_total` - Transport-specific errors
- `shadowcat_transport_reconnects_total` - Reconnection attempts

#### Interceptor Metrics
- `shadowcat_interceptor_actions_total` - Actions by type (continue, modify, block)
- `shadowcat_interceptor_processing_duration_seconds` - Processing time
- `shadowcat_interceptor_rules_matched_total` - Rule matches

#### Recording Metrics
- `shadowcat_recordings_active` - Current recordings
- `shadowcat_recording_size_bytes` - Recording size
- `shadowcat_recording_frames_total` - Frames recorded

### Tracing Spans

#### Request Lifecycle Spans
```
shadowcat.request
├── shadowcat.transport.receive
├── shadowcat.session.lookup
├── shadowcat.interceptor.chain
│   ├── shadowcat.interceptor.rule_match
│   └── shadowcat.interceptor.action
├── shadowcat.proxy.forward
│   ├── shadowcat.transport.connect
│   └── shadowcat.transport.send
└── shadowcat.recording.capture
```

#### Span Attributes
- `mcp.session_id` - Session identifier
- `mcp.method` - MCP method name
- `mcp.message_type` - request/response/notification
- `transport.type` - stdio/http/sse
- `proxy.direction` - forward/reverse
- `proxy.upstream` - Target server

## Implementation Plan

### Step 1: Add Dependencies (15 min)

```toml
# Cargo.toml
[dependencies]
# OpenTelemetry
opentelemetry = { version = "0.24", features = ["trace", "metrics"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.17", features = ["tonic", "metrics"] }
opentelemetry-prometheus = "0.17"
opentelemetry-semantic-conventions = "0.24"

# Prometheus
prometheus = "0.13"

# Tracing integration
tracing-opentelemetry = "0.25"
```

### Step 2: Create Telemetry Module (45 min)

```rust
// src/telemetry/mod.rs
pub mod tracer;
pub mod metrics;
pub mod middleware;

use opentelemetry::global;
use opentelemetry_sdk::runtime::Tokio;

pub struct TelemetryConfig {
    pub enabled: bool,
    pub service_name: String,
    pub otlp_endpoint: Option<String>,
    pub sampling_rate: f64,
    pub metrics_bind: SocketAddr,
}

pub fn init_telemetry(config: &TelemetryConfig) -> Result<TelemetryGuard> {
    if !config.enabled {
        return Ok(TelemetryGuard::noop());
    }
    
    // Initialize tracer
    let tracer = init_tracer(config)?;
    
    // Initialize metrics
    let meter = init_metrics(config)?;
    
    // Start Prometheus endpoint
    let metrics_server = start_metrics_server(config.metrics_bind)?;
    
    Ok(TelemetryGuard {
        _tracer: tracer,
        _meter: meter,
        metrics_server,
    })
}
```

### Step 3: Implement Tracing (1 hour)

```rust
// src/telemetry/tracer.rs
use opentelemetry::{trace::Tracer, KeyValue};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_tracer(config: &TelemetryConfig) -> Result<impl Tracer> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otlp_endpoint)
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(config.sampling_rate))
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", config.service_name.clone()),
                ]))
        )
        .install_batch(Tokio)?;
    
    // Set global tracer
    global::set_tracer_provider(tracer.provider());
    
    Ok(tracer)
}

// Instrument key functions
#[instrument(skip(transport), fields(transport.type = %transport.transport_type()))]
pub async fn forward_request(
    transport: &mut impl Transport,
    message: ProtocolMessage,
) -> Result<ProtocolMessage> {
    // Add span events
    tracing::info!(message.type = ?message.message_type(), "Forwarding request");
    
    // Implementation
    let span = tracing::Span::current();
    span.record("mcp.method", &message.method());
    
    transport.send(message).await
}
```

### Step 4: Implement Metrics (1 hour)

```rust
// src/telemetry/metrics.rs
use opentelemetry::metrics::{Counter, Histogram, UpDownCounter};
use prometheus::{Encoder, TextEncoder};

pub struct ProxyMetrics {
    requests_total: Counter<u64>,
    request_duration: Histogram<f64>,
    active_connections: UpDownCounter<i64>,
    errors_total: Counter<u64>,
    bytes_transferred: Counter<u64>,
}

impl ProxyMetrics {
    pub fn new(meter: &Meter) -> Self {
        Self {
            requests_total: meter
                .u64_counter("shadowcat_proxy_requests_total")
                .with_description("Total number of proxy requests")
                .init(),
                
            request_duration: meter
                .f64_histogram("shadowcat_proxy_request_duration_seconds")
                .with_description("Request processing duration")
                .with_unit(Unit::new("s"))
                .init(),
                
            active_connections: meter
                .i64_up_down_counter("shadowcat_proxy_active_connections")
                .with_description("Current active connections")
                .init(),
                
            // ... other metrics
        }
    }
    
    pub fn record_request(&self, transport: &str, direction: &str, duration: Duration) {
        self.requests_total.add(1, &[
            KeyValue::new("transport", transport),
            KeyValue::new("direction", direction),
        ]);
        
        self.request_duration.record(duration.as_secs_f64(), &[
            KeyValue::new("transport", transport),
        ]);
    }
}

// Prometheus endpoint
pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Response::builder()
        .header("Content-Type", encoder.format_type())
        .body(buffer)
        .unwrap()
}
```

### Step 5: Add Instrumentation Points (45 min)

```rust
// src/proxy/forward.rs
impl ForwardProxy {
    #[instrument(skip(self), fields(session.id = %session_id))]
    pub async fn handle_request(&mut self, message: ProtocolMessage) -> Result<ProtocolMessage> {
        let start = Instant::now();
        let span = Span::current();
        
        // Record metrics
        self.metrics.active_connections.add(1, &[]);
        
        let result = async {
            // Existing proxy logic
            span.add_event("Processing request", vec![
                KeyValue::new("mcp.method", message.method()),
            ]);
            
            self.transport.send(message).await
        }.await;
        
        // Record completion metrics
        self.metrics.active_connections.add(-1, &[]);
        self.metrics.record_request(
            self.transport.transport_type(),
            "forward",
            start.elapsed(),
        );
        
        result
    }
}

// src/session/manager.rs
impl SessionManager {
    #[instrument(skip(self))]
    pub async fn create_session(&self, id: SessionId) -> Result<Session> {
        self.metrics.sessions_created.add(1, &[]);
        self.metrics.sessions_active.add(1, &[]);
        
        // Create session
        let session = Session::new(id);
        
        tracing::info!(
            session.id = %id,
            "Created new session"
        );
        
        Ok(session)
    }
}
```

### Step 6: Create Middleware (30 min)

```rust
// src/telemetry/middleware.rs
use tower::{Layer, Service};

pub struct TelemetryLayer {
    metrics: Arc<ProxyMetrics>,
}

impl<S> Layer<S> for TelemetryLayer {
    type Service = TelemetryMiddleware<S>;
    
    fn layer(&self, service: S) -> Self::Service {
        TelemetryMiddleware {
            inner: service,
            metrics: self.metrics.clone(),
        }
    }
}

pub struct TelemetryMiddleware<S> {
    inner: S,
    metrics: Arc<ProxyMetrics>,
}

impl<S, B> Service<Request<B>> for TelemetryMiddleware<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    
    fn call(&mut self, req: Request<B>) -> Self::Future {
        let span = info_span!(
            "shadowcat.http.request",
            method = %req.method(),
            path = %req.uri().path(),
        );
        
        let start = Instant::now();
        let metrics = self.metrics.clone();
        
        ResponseFuture {
            future: self.inner.call(req),
            span,
            start,
            metrics,
        }
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_metrics_collection() {
    let metrics = ProxyMetrics::new(&meter);
    
    metrics.record_request("stdio", "forward", Duration::from_millis(100));
    
    // Verify metric was recorded
    let families = prometheus::gather();
    assert!(families.iter().any(|f| f.get_name() == "shadowcat_proxy_requests_total"));
}

#[tokio::test]
async fn test_tracing_span_creation() {
    let (tracer, provider) = init_test_tracer();
    
    forward_request(&mut transport, message).await.unwrap();
    
    let spans = provider.exported_spans();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "shadowcat.proxy.forward");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_metrics_endpoint() {
    let config = TelemetryConfig {
        enabled: true,
        metrics_bind: "127.0.0.1:9090".parse().unwrap(),
        // ...
    };
    
    let _guard = init_telemetry(&config).unwrap();
    
    // Make request to metrics endpoint
    let response = reqwest::get("http://127.0.0.1:9090/metrics").await.unwrap();
    assert_eq!(response.status(), 200);
    
    let body = response.text().await.unwrap();
    assert!(body.contains("shadowcat_proxy_requests_total"));
}
```

### Performance Tests

```rust
#[bench]
fn bench_with_telemetry(b: &mut Bencher) {
    let config = TelemetryConfig { enabled: true, /* ... */ };
    let _guard = init_telemetry(&config).unwrap();
    
    b.iter(|| {
        // Benchmark proxy operation with telemetry
    });
}

#[bench]
fn bench_without_telemetry(b: &mut Bencher) {
    let config = TelemetryConfig { enabled: false, /* ... */ };
    let _guard = init_telemetry(&config).unwrap();
    
    b.iter(|| {
        // Benchmark same operation without telemetry
    });
}
```

## Success Criteria

- [ ] OpenTelemetry tracer exports spans to OTLP endpoint
- [ ] Prometheus metrics available at /metrics endpoint
- [ ] All key operations instrumented with spans
- [ ] Core metrics collected for proxy, sessions, transport
- [ ] Performance overhead < 5% when enabled
- [ ] Zero overhead when disabled
- [ ] Sampling rate configuration works
- [ ] Integration with existing logging
- [ ] All existing tests pass

## Risk Mitigation

1. **Performance Impact**: Use sampling, lazy initialization, feature flags
2. **Dependency Size**: Make telemetry an optional feature
3. **Breaking Changes**: Keep telemetry fully optional and backward compatible
4. **Complex Setup**: Provide good defaults and examples

## Configuration Integration

Works with C.2 config file support:

```toml
[telemetry]
enabled = true
service_name = "shadowcat-prod"
otlp_endpoint = "http://otel-collector:4317"
sampling_rate = 0.1

[metrics]
enabled = true
bind = "0.0.0.0:9090"
path = "/metrics"
```

## Example Usage

```bash
# Enable telemetry via config
shadowcat --config prod.toml forward stdio -- server

# Enable via environment
SHADOWCAT_TELEMETRY_ENABLED=true \
SHADOWCAT_TELEMETRY_OTLP_ENDPOINT=http://localhost:4317 \
shadowcat forward stdio -- server

# Disable sampling for debugging
SHADOWCAT_TELEMETRY_SAMPLING_RATE=1.0 shadowcat forward stdio -- server
```

## Files to Create/Modify

1. **Create**: `src/telemetry/mod.rs` - Main telemetry module
2. **Create**: `src/telemetry/tracer.rs` - OpenTelemetry tracing
3. **Create**: `src/telemetry/metrics.rs` - Prometheus metrics
4. **Create**: `src/telemetry/middleware.rs` - HTTP middleware
5. **Modify**: `src/proxy/forward.rs` - Add instrumentation
6. **Modify**: `src/proxy/reverse.rs` - Add instrumentation
7. **Modify**: `src/session/manager.rs` - Add session metrics
8. **Modify**: `src/transport/mod.rs` - Add transport metrics
9. **Modify**: `src/main.rs` - Initialize telemetry
10. **Modify**: `Cargo.toml` - Add dependencies
11. **Create**: `examples/telemetry_example.rs` - Usage example

## Documentation to Add

1. Telemetry guide in `docs/telemetry.md`
2. Metrics reference with descriptions
3. Tracing span reference
4. Setup guides for common collectors (Jaeger, Prometheus, Grafana)
5. Dashboard examples for Grafana

## Performance Considerations

1. Use async-friendly collectors (batch export)
2. Implement circuit breaker for OTLP exports
3. Use bounded channels to prevent memory issues
4. Lazy metric creation (only when used)
5. Consider using `tracing` macros with compile-time filtering

## Future Enhancements

1. Custom metrics via configuration
2. Trace context propagation in MCP headers
3. Baggage support for cross-service metadata
4. Log correlation with trace IDs
5. Exemplar support for metrics-to-traces linking
6. Custom sampling strategies (head-based, tail-based)
7. Integration with distributed tracing systems