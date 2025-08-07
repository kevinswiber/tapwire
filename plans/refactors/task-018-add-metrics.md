# Task 018: Add Performance Metrics

## Overview
Implement comprehensive performance metrics collection and export in Prometheus format for production monitoring and observability.

## Context
From the [comprehensive review](../../reviews/shadowcat-comprehensive-review-2025-08-06.md), performance analysis identified the need for production monitoring. The system needs metrics to track performance, identify bottlenecks, and ensure the <5% overhead target is met.

## Scope
- **Files to modify**: Create `src/metrics/`, integrate throughout
- **Priority**: MEDIUM - Production monitoring
- **Time estimate**: 1.5 days

## Current Problem

### Missing Capabilities
- No performance metrics collection
- No Prometheus endpoint
- No visibility into proxy overhead
- No latency tracking
- No resource usage monitoring

## Implementation Plan

### Step 1: Define Core Metrics

```rust
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec,
    IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
    register_counter, register_counter_vec, register_gauge, register_gauge_vec,
    register_histogram, register_histogram_vec,
    register_int_counter, register_int_counter_vec,
};
use once_cell::sync::Lazy;

// Request metrics
pub static REQUEST_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "shadowcat_requests_total",
        "Total number of requests processed",
        &["method", "transport", "status"]
    ).unwrap()
});

pub static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "shadowcat_request_duration_seconds",
        "Request duration in seconds",
        &["method", "transport", "status"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    ).unwrap()
});

pub static REQUEST_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "shadowcat_request_size_bytes",
        "Request size in bytes",
        &["method", "transport"],
        vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]
    ).unwrap()
});

pub static RESPONSE_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "shadowcat_response_size_bytes",
        "Response size in bytes",
        &["method", "transport"],
        vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]
    ).unwrap()
});

// Session metrics
pub static ACTIVE_SESSIONS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "shadowcat_active_sessions",
        "Number of active sessions"
    ).unwrap()
});

pub static SESSION_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "shadowcat_session_duration_seconds",
        "Session duration in seconds",
        vec![1.0, 10.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]
    ).unwrap()
});

// Proxy metrics
pub static PROXY_LATENCY: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "shadowcat_proxy_latency_seconds",
        "Additional latency introduced by proxy",
        &["upstream"],
        vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1]
    ).unwrap()
});

pub static UPSTREAM_ERRORS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "shadowcat_upstream_errors_total",
        "Total upstream errors by type",
        &["upstream", "error_type"]
    ).unwrap()
});

// Circuit breaker metrics
pub static CIRCUIT_BREAKER_STATE: Lazy<IntGaugeVec> = Lazy::new(|| {
    register_int_gauge_vec!(
        "shadowcat_circuit_breaker_state",
        "Circuit breaker state (0=closed, 1=open, 2=half-open)",
        &["upstream"]
    ).unwrap()
});

// Rate limiting metrics
pub static RATE_LIMIT_EXCEEDED: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "shadowcat_rate_limit_exceeded_total",
        "Number of rate limit violations",
        &["limit_type", "client"]
    ).unwrap()
});

// Resource metrics
pub static MEMORY_USAGE: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "shadowcat_memory_usage_bytes",
        "Current memory usage in bytes"
    ).unwrap()
});

pub static CPU_USAGE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "shadowcat_cpu_usage_percent",
        "Current CPU usage percentage"
    ).unwrap()
});

pub static GOROUTINES: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "shadowcat_goroutines",
        "Number of active goroutines (tokio tasks)"
    ).unwrap()
});

// Tape metrics
pub static TAPE_SIZE: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "shadowcat_tape_size_bytes",
        "Size of recorded tapes",
        &["transport"],
        vec![1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0]
    ).unwrap()
});

pub static TAPE_OPERATIONS: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        "shadowcat_tape_operations_total",
        "Tape operations by type",
        &["operation"]
    ).unwrap()
});
```

### Step 2: Create Metrics Collector

```rust
use std::time::Instant;
use tokio::time::interval;

pub struct MetricsCollector {
    start_time: Instant,
    collection_interval: Duration,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            collection_interval: Duration::from_secs(10),
        }
    }
    
    pub fn start(self) {
        tokio::spawn(async move {
            let mut interval = interval(self.collection_interval);
            
            loop {
                interval.tick().await;
                self.collect_system_metrics();
            }
        });
    }
    
    fn collect_system_metrics(&self) {
        // Memory usage
        if let Ok(memory) = self.get_memory_usage() {
            MEMORY_USAGE.set(memory as i64);
        }
        
        // CPU usage
        if let Ok(cpu) = self.get_cpu_usage() {
            CPU_USAGE.set(cpu);
        }
        
        // Task count
        let task_count = tokio::runtime::Handle::current()
            .metrics()
            .num_alive_tasks();
        GOROUTINES.set(task_count as i64);
        
        // Uptime
        let uptime = self.start_time.elapsed().as_secs();
        UPTIME.set(uptime as i64);
    }
    
    #[cfg(target_os = "linux")]
    fn get_memory_usage(&self) -> Result<usize, Error> {
        use std::fs;
        
        let status = fs::read_to_string("/proc/self/status")?;
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: usize = parts[1].parse()?;
                    return Ok(kb * 1024);  // Convert to bytes
                }
            }
        }
        
        Err(Error::MetricNotAvailable)
    }
    
    #[cfg(not(target_os = "linux"))]
    fn get_memory_usage(&self) -> Result<usize, Error> {
        // Fallback for non-Linux systems
        Ok(0)
    }
    
    fn get_cpu_usage(&self) -> Result<f64, Error> {
        // CPU usage calculation (platform-specific)
        // This is a simplified version
        Ok(0.0)
    }
}
```

### Step 3: Instrument Request Processing

```rust
pub struct MetricsMiddleware;

impl MetricsMiddleware {
    pub async fn track_request<F, Fut>(
        &self,
        method: &str,
        transport: &str,
        f: F,
    ) -> Result<Response, Error>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Response, Error>>,
    {
        let start = Instant::now();
        
        // Execute the actual request
        let result = f().await;
        
        let duration = start.elapsed().as_secs_f64();
        let status = match &result {
            Ok(resp) => resp.status().as_str(),
            Err(_) => "error",
        };
        
        // Record metrics
        REQUEST_COUNTER
            .with_label_values(&[method, transport, status])
            .inc();
        
        REQUEST_DURATION
            .with_label_values(&[method, transport, status])
            .observe(duration);
        
        result
    }
}

// Use in proxy
impl ForwardProxy {
    pub async fn handle_request(&self, request: Request) -> Result<Response, Error> {
        let method = request.method.clone();
        let transport = "stdio";  // or "http"
        
        MetricsMiddleware.track_request(
            &method,
            transport,
            || async {
                // Record request size
                if let Some(size) = request.content_length() {
                    REQUEST_SIZE
                        .with_label_values(&[&method, transport])
                        .observe(size as f64);
                }
                
                // Measure proxy overhead
                let proxy_start = Instant::now();
                let response = self.forward_to_upstream(request).await?;
                let proxy_latency = proxy_start.elapsed().as_secs_f64();
                
                PROXY_LATENCY
                    .with_label_values(&[&self.upstream_url])
                    .observe(proxy_latency);
                
                // Record response size
                if let Some(size) = response.content_length() {
                    RESPONSE_SIZE
                        .with_label_values(&[&method, transport])
                        .observe(size as f64);
                }
                
                Ok(response)
            }
        ).await
    }
}
```

### Step 4: Add Prometheus Endpoint

```rust
use axum::{
    Router,
    routing::get,
    response::Response,
    http::StatusCode,
};
use prometheus::{Encoder, TextEncoder};

pub fn metrics_router() -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
}

async fn metrics_handler() -> Result<Response<String>, StatusCode> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let output = String::from_utf8(buffer)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", encoder.format_type())
        .body(output)
        .unwrap())
}

async fn health_handler() -> StatusCode {
    // Basic health check
    StatusCode::OK
}

async fn ready_handler() -> StatusCode {
    // Check if service is ready to accept traffic
    if is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

fn is_ready() -> bool {
    // Check upstream connectivity, etc.
    true
}
```

### Step 5: Add Custom Metrics

```rust
pub trait MetricsRecorder {
    fn record_custom(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

pub struct CustomMetrics {
    metrics: Arc<DashMap<String, Histogram>>,
}

impl CustomMetrics {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
        }
    }
    
    pub fn record(&self, metric_name: &str, value: f64) {
        let histogram = self.metrics.entry(metric_name.to_string())
            .or_insert_with(|| {
                register_histogram!(
                    metric_name,
                    format!("Custom metric: {}", metric_name)
                ).unwrap()
            });
        
        histogram.observe(value);
    }
}

// Allow users to define custom metrics
pub fn register_custom_metric(
    name: &str,
    help: &str,
    buckets: Vec<f64>,
) -> Result<Histogram, Error> {
    let histogram = Histogram::with_opts(
        HistogramOpts::new(name, help)
            .buckets(buckets)
    )?;
    
    prometheus::register(Box::new(histogram.clone()))?;
    Ok(histogram)
}
```

### Step 6: Add Performance Benchmarking

```rust
pub struct PerformanceBenchmark {
    baseline_latency: Duration,
    max_overhead_percent: f32,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            baseline_latency: Duration::from_micros(100),
            max_overhead_percent: 5.0,
        }
    }
    
    pub async fn measure_overhead(&self) -> BenchmarkResult {
        // Measure direct call
        let direct_start = Instant::now();
        let _ = self.direct_call().await;
        let direct_duration = direct_start.elapsed();
        
        // Measure through proxy
        let proxy_start = Instant::now();
        let _ = self.proxy_call().await;
        let proxy_duration = proxy_start.elapsed();
        
        // Calculate overhead
        let overhead = proxy_duration.saturating_sub(direct_duration);
        let overhead_percent = (overhead.as_secs_f64() / direct_duration.as_secs_f64()) * 100.0;
        
        BenchmarkResult {
            direct_latency: direct_duration,
            proxy_latency: proxy_duration,
            overhead: overhead,
            overhead_percent: overhead_percent as f32,
            passes_target: overhead_percent <= self.max_overhead_percent as f64,
        }
    }
}

#[derive(Debug)]
pub struct BenchmarkResult {
    pub direct_latency: Duration,
    pub proxy_latency: Duration,
    pub overhead: Duration,
    pub overhead_percent: f32,
    pub passes_target: bool,
}
```

### Step 7: Add Metrics Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub port: u16,
    pub path: String,
    pub collection_interval_secs: u64,
    pub histogram_buckets: Option<Vec<f64>>,
    pub cardinality_limit: Option<usize>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9090,
            path: "/metrics".to_string(),
            collection_interval_secs: 10,
            histogram_buckets: None,
            cardinality_limit: Some(10000),
        }
    }
}
```

## Testing Strategy

### Performance Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();
        
        // Simulate requests
        for i in 0..100 {
            REQUEST_COUNTER
                .with_label_values(&["ping", "stdio", "200"])
                .inc();
            
            REQUEST_DURATION
                .with_label_values(&["ping", "stdio", "200"])
                .observe(0.01 * i as f64);
        }
        
        // Get metrics
        let families = prometheus::gather();
        
        // Verify metrics exist
        let request_count = families.iter()
            .find(|f| f.get_name() == "shadowcat_requests_total")
            .unwrap();
        
        assert!(request_count.get_metric().len() > 0);
    }
    
    #[tokio::test]
    async fn test_overhead_target() {
        let benchmark = PerformanceBenchmark::new();
        let result = benchmark.measure_overhead().await;
        
        assert!(
            result.passes_target,
            "Proxy overhead {}% exceeds 5% target",
            result.overhead_percent
        );
    }
    
    #[tokio::test]
    async fn test_prometheus_endpoint() {
        let app = metrics_router();
        
        let response = app
            .oneshot(Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();
        
        assert_eq!(response.status(), StatusCode::OK);
        
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        
        assert!(text.contains("shadowcat_requests_total"));
    }
}
```

### Load Test with Metrics

```bash
#!/bin/bash
# Start shadowcat with metrics
./target/release/shadowcat forward stdio --metrics-port 9090 &
PID=$!

# Wait for startup
sleep 2

# Run load test
ab -n 10000 -c 100 http://localhost:8080/

# Query metrics
curl -s http://localhost:9090/metrics | grep shadowcat_

# Check overhead
curl -s http://localhost:9090/metrics | \
    grep shadowcat_proxy_latency_seconds | \
    awk '{print "Proxy overhead:", $2}'

kill $PID
```

## Validation

### Pre-check
```bash
# No metrics endpoint
curl http://localhost:9090/metrics  # Should fail
```

### Post-check
```bash
# Metrics available
curl http://localhost:9090/metrics | grep shadowcat_

# Verify key metrics
curl -s http://localhost:9090/metrics | grep -E \
    "shadowcat_requests_total|shadowcat_request_duration_seconds|shadowcat_proxy_latency_seconds"

# Check overhead is under 5%
./benchmark.sh  # Custom benchmark script
```

## Success Criteria

- [ ] Core metrics implemented (requests, latency, errors)
- [ ] Prometheus endpoint available
- [ ] System metrics collected (memory, CPU)
- [ ] Session metrics tracked
- [ ] Circuit breaker state exposed
- [ ] Rate limiting metrics available
- [ ] Proxy overhead measured and < 5%
- [ ] Health/ready endpoints working
- [ ] Metrics configurable
- [ ] All tests pass

## Grafana Dashboard Example

```json
{
  "dashboard": {
    "title": "Shadowcat Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(shadowcat_requests_total[5m])"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, shadowcat_request_duration_seconds)"
          }
        ]
      },
      {
        "title": "Proxy Overhead",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, shadowcat_proxy_latency_seconds)"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(shadowcat_upstream_errors_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Performance Considerations

1. **Cardinality control** - Limit label combinations
2. **Histogram buckets** - Choose appropriate bucket sizes
3. **Collection frequency** - Balance accuracy vs overhead
4. **Metric storage** - Consider metric retention

## Integration Points

- All request handlers need instrumentation
- Session manager tracks active sessions
- Circuit breaker exports state
- Rate limiter tracks violations
- Audit logger can export security metrics

## Notes

- Essential for production monitoring
- Keep cardinality under control (max 10K series)
- Consider adding trace sampling for detailed debugging
- May want to support other formats (StatsD, OpenTelemetry)