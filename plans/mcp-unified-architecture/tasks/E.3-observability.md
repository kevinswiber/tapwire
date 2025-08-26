# Task E.3: Observability and Metrics

## Objective
Implement comprehensive observability using OpenTelemetry with Prometheus as the default exporter, following shadowcat's established patterns.

## Background
Per discussion: "We do prometheus by default using opentelemetry, opentelemetry_sdk, and opentelemetry-prometheus. We have opentelemetry-otlp optional because we have an `otlp` feature flag, as it brings in tonic."

## Key Requirements

### 1. Dependencies Structure
```toml
[dependencies]
# Core observability
opentelemetry = { version = "0.24", features = ["metrics", "trace"] }
opentelemetry_sdk = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-prometheus = "0.17"
prometheus = "0.13"

# Optional OTLP support
opentelemetry-otlp = { version = "0.17", optional = true, features = ["grpc-tonic"] }
tonic = { version = "0.12", optional = true }

[features]
default = ["metrics"]
metrics = ["dep:opentelemetry", "dep:opentelemetry_sdk", "dep:opentelemetry-prometheus"]
otlp = ["dep:opentelemetry-otlp", "dep:tonic", "metrics"]
```

### 2. Metrics Registry
```rust
use opentelemetry::metrics::{Counter, Histogram, Meter, UpDownCounter};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use std::sync::OnceLock;

pub struct Metrics {
    // Connection metrics
    pub connections_total: Counter<u64>,
    pub connections_active: UpDownCounter<i64>,
    pub connection_duration: Histogram<f64>,
    pub connection_errors: Counter<u64>,
    
    // Session metrics
    pub sessions_created: Counter<u64>,
    pub sessions_active: UpDownCounter<i64>,
    pub sessions_evicted: Counter<u64>,
    pub session_duration: Histogram<f64>,
    pub session_heartbeat_checks: Counter<u64>,
    pub session_dead_detected: Counter<u64>,
    
    // Request/Response metrics
    pub requests_total: Counter<u64>,
    pub request_duration: Histogram<f64>,
    pub request_size: Histogram<u64>,
    pub response_size: Histogram<u64>,
    
    // Interceptor metrics
    pub interceptor_duration: Histogram<f64>,
    pub interceptor_errors: Counter<u64>,
    pub interceptor_actions: Counter<u64>,
    
    // Pool metrics (Client-side)
    pub pool_connections_created: Counter<u64>,
    pub pool_connections_reused: Counter<u64>,
    pub pool_connections_idle: UpDownCounter<i64>,
    pub pool_wait_duration: Histogram<f64>,
    
    // Resource metrics
    pub memory_usage: UpDownCounter<i64>,
    pub task_spawns: Counter<u64>,
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

impl Metrics {
    pub fn global() -> &'static Metrics {
        METRICS.get_or_init(|| {
            let meter = opentelemetry::global::meter("mcp");
            
            Metrics {
                // Connection metrics
                connections_total: meter
                    .u64_counter("mcp.connections.total")
                    .with_description("Total number of connections")
                    .init(),
                    
                connections_active: meter
                    .i64_up_down_counter("mcp.connections.active")
                    .with_description("Number of active connections")
                    .init(),
                    
                connection_duration: meter
                    .f64_histogram("mcp.connection.duration")
                    .with_description("Connection duration in seconds")
                    .with_unit("s")
                    .init(),
                    
                // Session metrics
                sessions_created: meter
                    .u64_counter("mcp.sessions.created")
                    .with_description("Total sessions created")
                    .init(),
                    
                sessions_active: meter
                    .i64_up_down_counter("mcp.sessions.active")
                    .with_description("Number of active sessions")
                    .init(),
                    
                session_duration: meter
                    .f64_histogram("mcp.session.duration")
                    .with_description("Session duration in seconds")
                    .with_unit("s")
                    .init(),
                    
                // Request metrics
                requests_total: meter
                    .u64_counter("mcp.requests.total")
                    .with_description("Total number of requests")
                    .init(),
                    
                request_duration: meter
                    .f64_histogram("mcp.request.duration")
                    .with_description("Request processing duration")
                    .with_unit("s")
                    .init(),
                    
                // ... initialize all metrics
            }
        })
    }
}
```

### 3. Metrics Initialization
```rust
pub fn init_metrics(config: &MetricsConfig) -> Result<(), Error> {
    match config.exporter {
        MetricsExporter::Prometheus => init_prometheus(config)?,
        #[cfg(feature = "otlp")]
        MetricsExporter::Otlp => init_otlp(config)?,
        MetricsExporter::None => {},
    }
    
    Ok(())
}

fn init_prometheus(config: &MetricsConfig) -> Result<(), Error> {
    use opentelemetry_prometheus::PrometheusExporter;
    use prometheus::{Encoder, TextEncoder};
    
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(prometheus::Registry::new())
        .build()?;
    
    let provider = SdkMeterProvider::builder()
        .with_reader(exporter)
        .build();
    
    opentelemetry::global::set_meter_provider(provider);
    
    // Start metrics endpoint
    if let Some(bind_addr) = &config.prometheus_bind {
        tokio::spawn(serve_metrics(bind_addr.clone(), exporter));
    }
    
    Ok(())
}

#[cfg(feature = "otlp")]
fn init_otlp(config: &MetricsConfig) -> Result<(), Error> {
    use opentelemetry_otlp::WithExportConfig;
    
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint);
    
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(exporter)
        .build()?;
    
    opentelemetry::global::set_meter_provider(provider);
    
    Ok(())
}

async fn serve_metrics(bind_addr: String, exporter: PrometheusExporter) {
    use axum::{Router, routing::get, response::Response};
    
    let app = Router::new()
        .route("/metrics", get(move || async move {
            let mut buffer = Vec::new();
            let encoder = TextEncoder::new();
            let metric_families = exporter.registry().gather();
            encoder.encode(&metric_families, &mut buffer).unwrap();
            
            Response::builder()
                .header("Content-Type", encoder.format_type())
                .body(String::from_utf8(buffer).unwrap())
                .unwrap()
        }));
    
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind metrics endpoint");
    
    tracing::info!("Metrics endpoint listening on {}", bind_addr);
    
    axum::serve(listener, app)
        .await
        .expect("Metrics server failed");
}
```

### 4. Instrumentation Points

#### Connection Lifecycle
```rust
impl Server {
    async fn handle_connection<C: Connection>(&self, conn: C) -> Result<(), Error> {
        let start = Instant::now();
        let metrics = Metrics::global();
        
        metrics.connections_total.add(1, &[]);
        metrics.connections_active.add(1, &[]);
        
        let _guard = defer(|| {
            metrics.connections_active.add(-1, &[]);
            metrics.connection_duration.record(
                start.elapsed().as_secs_f64(),
                &[KeyValue::new("status", "closed")]
            );
        });
        
        // Connection handling...
    }
}
```

#### Session Operations
```rust
impl SessionManager {
    pub async fn create_session(&self, id: SessionId) -> Result<Session, Error> {
        let metrics = Metrics::global();
        metrics.sessions_created.add(1, &[]);
        metrics.sessions_active.add(1, &[]);
        
        // Create session...
    }
    
    pub async fn check_session_liveness(&self) {
        let metrics = Metrics::global();
        metrics.session_heartbeat_checks.add(1, &[]);
        
        // Check liveness...
        
        if !alive {
            metrics.session_dead_detected.add(1, &[]);
        }
    }
}
```

#### Interceptor Chain
```rust
impl InterceptorChain {
    pub async fn process_request(&self, request: JsonRpcRequest) -> Result<JsonRpcRequest, Error> {
        for interceptor in &self.interceptors {
            let start = Instant::now();
            
            let result = interceptor.intercept_request(&mut request, &context).await;
            
            let metrics = Metrics::global();
            metrics.interceptor_duration.record(
                start.elapsed().as_secs_f64(),
                &[
                    KeyValue::new("interceptor", interceptor.name()),
                    KeyValue::new("phase", "request"),
                ]
            );
            
            if let Err(e) = &result {
                metrics.interceptor_errors.add(
                    1,
                    &[
                        KeyValue::new("interceptor", interceptor.name()),
                        KeyValue::new("error_type", e.variant_name()),
                    ]
                );
            }
            
            // Process result...
        }
    }
}
```

#### Pool Metrics (Client)
```rust
impl<T: PoolableResource> Pool<T> {
    pub async fn acquire(&self) -> Result<PooledConnection<T>, Error> {
        let start = Instant::now();
        let metrics = Metrics::global();
        
        // Acquire connection...
        
        metrics.pool_wait_duration.record(
            start.elapsed().as_secs_f64(),
            &[KeyValue::new("pool", self.name)]
        );
        
        if created_new {
            metrics.pool_connections_created.add(1, &[]);
        } else {
            metrics.pool_connections_reused.add(1, &[]);
        }
        
        Ok(conn)
    }
}
```

### 5. Distributed Tracing (Optional)
```rust
#[cfg(feature = "tracing")]
pub fn init_tracing(config: &TracingConfig) -> Result<(), Error> {
    use opentelemetry::trace::TracerProvider;
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    let tracer = match config.exporter {
        TracingExporter::Otlp => {
            opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(
                    opentelemetry_otlp::new_exporter()
                        .tonic()
                        .with_endpoint(&config.otlp_endpoint)
                )
                .install_batch(opentelemetry_sdk::runtime::Tokio)?
        },
        TracingExporter::Jaeger => {
            // Jaeger support if needed
            todo!()
        },
    };
    
    let telemetry_layer = OpenTelemetryLayer::new(tracer);
    
    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}
```

### 6. Configuration
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    
    /// Metrics exporter
    pub exporter: MetricsExporter,
    
    /// Prometheus bind address (e.g., "0.0.0.0:9090")
    pub prometheus_bind: Option<String>,
    
    /// OTLP endpoint (when using OTLP exporter)
    #[cfg(feature = "otlp")]
    pub otlp_endpoint: String,
    
    /// Metric collection interval
    pub collection_interval: Duration,
}

#[derive(Debug, Clone, Deserialize)]
pub enum MetricsExporter {
    Prometheus,
    #[cfg(feature = "otlp")]
    Otlp,
    None,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            exporter: MetricsExporter::Prometheus,
            prometheus_bind: Some("0.0.0.0:9090".to_string()),
            #[cfg(feature = "otlp")]
            otlp_endpoint: "http://localhost:4317".to_string(),
            collection_interval: Duration::from_secs(10),
        }
    }
}
```

## Implementation Steps

1. **Set up dependencies** (30 min)
   - Add OpenTelemetry crates
   - Configure features for optional OTLP

2. **Create metrics registry** (1 hour)
   - Define all metric types
   - Implement global accessor
   - Add metric descriptions and units

3. **Implement Prometheus exporter** (1 hour)
   - Set up Prometheus registry
   - Create metrics endpoint
   - Test with Prometheus scraping

4. **Add OTLP support** (1 hour)
   - Conditional compilation with feature flag
   - Configure OTLP exporter
   - Handle tonic dependency

5. **Instrument connection lifecycle** (1 hour)
   - Connection counts and duration
   - Error tracking
   - Graceful cleanup

6. **Instrument sessions** (1 hour)
   - Session creation/eviction
   - Heartbeat metrics
   - Memory pressure events

7. **Instrument interceptors** (30 min)
   - Execution duration by interceptor
   - Error rates by type
   - Action distributions

8. **Add pool metrics** (30 min)
   - Connection creation vs reuse
   - Wait times
   - Pool saturation

## Testing Strategy

1. **Unit Tests**
   - Metric registration
   - Counter/histogram updates
   - Label cardinality

2. **Integration Tests**
   - Prometheus endpoint scraping
   - OTLP export (with mock collector)
   - Metric accuracy under load

3. **Load Testing**
   - Metric overhead measurement
   - Cardinality explosion prevention
   - Memory usage with metrics

## Success Criteria

- [ ] Prometheus endpoint serves metrics
- [ ] All key operations instrumented
- [ ] Metric overhead < 2% CPU
- [ ] OTLP export works when enabled
- [ ] No metric cardinality explosion
- [ ] Grafana dashboard ready

## Risk Mitigation

1. **High Cardinality**: Limit label values, use bounded sets
2. **Performance Impact**: Batch metric updates, use atomic operations
3. **Memory Growth**: Implement metric garbage collection

## Dependencies
- Core MCP implementation must be in place
- Shadowcat patterns as reference

## Estimated Duration
6.5 hours

## Notes
- Follow shadowcat's metric naming conventions
- Consider adding custom Grafana dashboards
- May need to clean up shadowcat's OTLP feature flag usage
- Prometheus is default, OTLP is opt-in to avoid tonic dependency