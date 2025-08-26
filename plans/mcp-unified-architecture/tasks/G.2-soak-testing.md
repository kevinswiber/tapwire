# Task G.2: Soak Testing and Long-Running Validation

## Objective
Implement comprehensive soak tests that run the system under sustained load for extended periods to detect memory leaks, resource exhaustion, and performance degradation.

## Background
From Gemini's review: "Include a testing phase for long-running soak tests. These tests run the proxy under a sustained, moderate load for an extended period (e.g., 24-48 hours) to detect memory leaks, resource handle leaks, and performance degradation over time."

## Key Requirements

### 1. Soak Test Framework
```rust
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use metrics_exporter_prometheus::PrometheusHandle;

pub struct SoakTestRunner {
    duration: Duration,
    config: SoakTestConfig,
    metrics: MetricsCollector,
    anomaly_detector: AnomalyDetector,
}

#[derive(Debug, Clone)]
pub struct SoakTestConfig {
    /// Target requests per second
    pub target_rps: u32,
    
    /// Number of concurrent clients
    pub concurrent_clients: usize,
    
    /// Message size distribution
    pub message_sizes: MessageSizeDistribution,
    
    /// Mix of operations to perform
    pub operation_mix: OperationMix,
    
    /// Resource monitoring interval
    pub monitoring_interval: Duration,
    
    /// Memory growth threshold (MB/hour)
    pub memory_growth_threshold: f64,
    
    /// Performance degradation threshold (% slower)
    pub perf_degradation_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct OperationMix {
    pub simple_requests: f64,      // 60%
    pub batch_requests: f64,       // 20%
    pub streaming_requests: f64,   // 10%
    pub error_requests: f64,       // 5%
    pub slow_requests: f64,        // 5%
}

pub struct SoakTestResults {
    pub duration: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub memory_samples: Vec<MemorySample>,
    pub latency_samples: Vec<LatencySample>,
    pub resource_leaks: Vec<ResourceLeak>,
    pub performance_anomalies: Vec<PerformanceAnomaly>,
    pub final_verdict: TestVerdict,
}

#[derive(Debug)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub heap_allocated: usize,
    pub heap_used: usize,
    pub stack_used: usize,
    pub handle_count: usize,
}

#[derive(Debug)]
pub struct ResourceLeak {
    pub resource_type: String,
    pub growth_rate: f64,
    pub evidence: Vec<String>,
}

impl SoakTestRunner {
    pub async fn run(&mut self) -> Result<SoakTestResults, Error> {
        let start = Instant::now();
        let mut results = SoakTestResults::default();
        
        // Start server
        let server = self.start_test_server().await?;
        
        // Start monitoring task
        let monitor_handle = self.start_monitoring(server.clone()).await;
        
        // Start client load generators
        let load_handles = self.start_load_generators(server.clone()).await;
        
        // Run for specified duration
        let test_complete = tokio::time::sleep(self.duration);
        
        tokio::select! {
            _ = test_complete => {
                tracing::info!("Soak test duration reached");
            },
            anomaly = self.anomaly_detector.critical_anomaly() => {
                tracing::error!(?anomaly, "Critical anomaly detected, stopping test");
                results.final_verdict = TestVerdict::Failed(anomaly);
            }
        }
        
        // Stop load generators
        for handle in load_handles {
            handle.abort();
        }
        
        // Collect final metrics
        results.duration = start.elapsed();
        results.memory_samples = monitor_handle.await?;
        results.resource_leaks = self.detect_leaks(&results.memory_samples)?;
        results.performance_anomalies = self.detect_anomalies(&results.latency_samples)?;
        
        if results.resource_leaks.is_empty() && results.performance_anomalies.is_empty() {
            results.final_verdict = TestVerdict::Passed;
        }
        
        Ok(results)
    }
    
    async fn start_monitoring(&self, server: Arc<Server>) -> JoinHandle<Vec<MemorySample>> {
        let interval = self.config.monitoring_interval;
        
        tokio::spawn(async move {
            let mut samples = Vec::new();
            let mut ticker = tokio::time::interval(interval);
            
            loop {
                ticker.tick().await;
                
                let sample = MemorySample {
                    timestamp: Instant::now(),
                    heap_allocated: get_heap_allocated(),
                    heap_used: get_heap_used(),
                    stack_used: get_stack_used(),
                    handle_count: get_handle_count(),
                };
                
                samples.push(sample);
                
                // Check for immediate issues
                if let Some(last_hour) = samples.windows(60).last() {
                    let growth = calculate_memory_growth(last_hour);
                    if growth > self.config.memory_growth_threshold {
                        tracing::warn!(
                            growth_mb_per_hour = growth,
                            "Excessive memory growth detected"
                        );
                    }
                }
            }
            
            samples
        })
    }
}
```

### 2. Memory Leak Detection
```rust
pub struct LeakDetector {
    baseline_snapshot: MemorySnapshot,
    growth_threshold: f64,
}

impl LeakDetector {
    pub fn analyze(&self, samples: &[MemorySample]) -> Vec<ResourceLeak> {
        let mut leaks = Vec::new();
        
        // Analyze heap growth
        if let Some(heap_leak) = self.analyze_heap_growth(samples) {
            leaks.push(heap_leak);
        }
        
        // Analyze handle growth
        if let Some(handle_leak) = self.analyze_handle_growth(samples) {
            leaks.push(handle_leak);
        }
        
        // Analyze specific allocations
        if let Some(alloc_leaks) = self.analyze_allocations() {
            leaks.extend(alloc_leaks);
        }
        
        leaks
    }
    
    fn analyze_heap_growth(&self, samples: &[MemorySample]) -> Option<ResourceLeak> {
        // Linear regression to detect consistent growth
        let points: Vec<(f64, f64)> = samples
            .iter()
            .map(|s| (s.timestamp.as_secs_f64(), s.heap_used as f64))
            .collect();
        
        let (slope, r_squared) = linear_regression(&points);
        
        // Slope is bytes/second, convert to MB/hour
        let growth_mb_per_hour = slope * 3600.0 / (1024.0 * 1024.0);
        
        if growth_mb_per_hour > self.growth_threshold && r_squared > 0.8 {
            Some(ResourceLeak {
                resource_type: "Heap Memory".to_string(),
                growth_rate: growth_mb_per_hour,
                evidence: vec![
                    format!("Growth rate: {:.2} MB/hour", growth_mb_per_hour),
                    format!("R²: {:.3}", r_squared),
                ],
            })
        } else {
            None
        }
    }
    
    fn analyze_allocations(&self) -> Option<Vec<ResourceLeak>> {
        // Use jemalloc profiling to find growing allocations
        #[cfg(feature = "jemalloc")]
        {
            let prof = jemalloc_ctl::profiling::prof::active::mib().unwrap();
            prof.set(true).unwrap();
            
            // Dump heap profile
            let dump = jemalloc_ctl::profiling::prof::dump::mib().unwrap();
            dump.set("heap.prof").unwrap();
            
            // Analyze profile for growing allocations
            let growing = analyze_heap_profile("heap.prof");
            
            if !growing.is_empty() {
                return Some(
                    growing
                        .into_iter()
                        .map(|alloc| ResourceLeak {
                            resource_type: "Allocation".to_string(),
                            growth_rate: alloc.growth_rate,
                            evidence: vec![
                                format!("Stack trace: {}", alloc.stack_trace),
                                format!("Size: {} bytes", alloc.size),
                            ],
                        })
                        .collect()
                );
            }
        }
        
        None
    }
}

// Platform-specific resource monitoring
#[cfg(target_os = "linux")]
fn get_process_stats() -> ProcessStats {
    use procfs::process::Process;
    
    let proc = Process::myself().unwrap();
    let stat = proc.stat().unwrap();
    let status = proc.status().unwrap();
    
    ProcessStats {
        virt_mem: status.vmsize.unwrap_or(0),
        rss_mem: status.vmrss.unwrap_or(0),
        threads: stat.num_threads,
        fds: proc.fd_count().unwrap_or(0),
    }
}

#[cfg(target_os = "macos")]
fn get_process_stats() -> ProcessStats {
    use mach2::task_info::{task_basic_info_data_t, TASK_BASIC_INFO};
    use mach2::task::{task_info, mach_task_self};
    
    unsafe {
        let mut info: task_basic_info_data_t = std::mem::zeroed();
        let mut count = std::mem::size_of::<task_basic_info_data_t>() as u32;
        
        task_info(
            mach_task_self(),
            TASK_BASIC_INFO,
            &mut info as *mut _ as *mut _,
            &mut count,
        );
        
        ProcessStats {
            virt_mem: info.virtual_size as usize,
            rss_mem: info.resident_size as usize,
            threads: info.thread_count as usize,
            fds: count_open_fds(),
        }
    }
}
```

### 3. Performance Degradation Detection
```rust
pub struct PerformanceDegradationDetector {
    baseline_latencies: LatencyBaseline,
    degradation_threshold: f64,
}

impl PerformanceDegradationDetector {
    pub fn detect(&self, samples: &[LatencySample]) -> Vec<PerformanceAnomaly> {
        let mut anomalies = Vec::new();
        
        // Group samples by hour
        let hourly_buckets = group_by_hour(samples);
        
        for (hour, hour_samples) in hourly_buckets.iter().enumerate() {
            let p50 = percentile(&hour_samples, 50);
            let p95 = percentile(&hour_samples, 95);
            let p99 = percentile(&hour_samples, 99);
            
            // Compare with baseline
            let p50_degradation = (p50 - self.baseline_latencies.p50) / self.baseline_latencies.p50;
            let p95_degradation = (p95 - self.baseline_latencies.p95) / self.baseline_latencies.p95;
            let p99_degradation = (p99 - self.baseline_latencies.p99) / self.baseline_latencies.p99;
            
            if p95_degradation > self.degradation_threshold {
                anomalies.push(PerformanceAnomaly {
                    timestamp: hour * 3600,
                    metric: "p95_latency".to_string(),
                    baseline_value: self.baseline_latencies.p95,
                    observed_value: p95,
                    degradation_percent: p95_degradation * 100.0,
                });
            }
        }
        
        anomalies
    }
}
```

### 4. Load Generation
```rust
pub struct LoadGenerator {
    target_rps: u32,
    client: Client,
    operation_mix: OperationMix,
    stats: Arc<RwLock<LoadStats>>,
}

impl LoadGenerator {
    pub async fn run(&self, duration: Duration) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000 / self.target_rps as u64));
        let deadline = Instant::now() + duration;
        
        while Instant::now() < deadline {
            interval.tick().await;
            
            let operation = self.select_operation();
            let start = Instant::now();
            
            let result = match operation {
                Operation::Simple => self.simple_request().await,
                Operation::Batch => self.batch_request().await,
                Operation::Streaming => self.streaming_request().await,
                Operation::Error => self.error_request().await,
                Operation::Slow => self.slow_request().await,
            };
            
            let latency = start.elapsed();
            
            let mut stats = self.stats.write().await;
            stats.record_request(latency, result.is_ok());
        }
    }
    
    async fn simple_request(&self) -> Result<JsonRpcResponse, Error> {
        self.client.request("test.echo", json!({"message": "hello"})).await
    }
    
    async fn batch_request(&self) -> Result<Vec<JsonRpcResponse>, Error> {
        let requests = (0..10)
            .map(|i| JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                method: "test.echo".to_string(),
                params: Some(json!({"message": format!("batch_{}", i)})),
                id: Some(JsonRpcId::Number(i)),
            })
            .collect();
        
        self.client.batch_request(requests).await
    }
    
    async fn streaming_request(&self) -> Result<(), Error> {
        let mut stream = self.client.stream_request("test.stream", json!({})).await?;
        
        while let Some(event) = stream.next().await {
            // Process streaming events
        }
        
        Ok(())
    }
}
```

### 5. Soak Test Scenarios
```rust
#[cfg(test)]
mod soak_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Run manually with: cargo test --ignored soak_24h
    async fn soak_24h_moderate_load() {
        let config = SoakTestConfig {
            target_rps: 100,
            concurrent_clients: 50,
            message_sizes: MessageSizeDistribution::realistic(),
            operation_mix: OperationMix::realistic(),
            monitoring_interval: Duration::from_secs(60),
            memory_growth_threshold: 10.0, // 10 MB/hour
            perf_degradation_threshold: 0.20, // 20% slower
        };
        
        let mut runner = SoakTestRunner::new(Duration::from_secs(24 * 3600), config);
        let results = runner.run().await.unwrap();
        
        // Assert no memory leaks
        assert!(
            results.resource_leaks.is_empty(),
            "Memory leaks detected: {:?}",
            results.resource_leaks
        );
        
        // Assert no performance degradation
        assert!(
            results.performance_anomalies.is_empty(),
            "Performance degradation detected: {:?}",
            results.performance_anomalies
        );
        
        // Generate report
        results.generate_report("soak_24h_report.html").await.unwrap();
    }
    
    #[tokio::test]
    #[ignore] // Run manually
    async fn soak_48h_with_reconnects() {
        let config = SoakTestConfig {
            target_rps: 50,
            concurrent_clients: 100,
            ..Default::default()
        };
        
        let mut runner = SoakTestRunner::new(Duration::from_secs(48 * 3600), config);
        
        // Add reconnection chaos
        runner.add_chaos(ChaosConfig {
            disconnect_probability: 0.01, // 1% chance per minute
            reconnect_delay: Duration::from_secs(5),
        });
        
        let results = runner.run().await.unwrap();
        
        // Verify session cleanup
        assert_eq!(
            results.orphaned_sessions, 0,
            "Orphaned sessions found"
        );
    }
    
    #[tokio::test]
    #[ignore]
    async fn soak_memory_stress() {
        // Test with large messages to stress memory management
        let config = SoakTestConfig {
            target_rps: 10,
            concurrent_clients: 20,
            message_sizes: MessageSizeDistribution {
                small: 0.1,    // 10% small (< 1KB)
                medium: 0.3,   // 30% medium (1KB - 100KB)
                large: 0.5,    // 50% large (100KB - 10MB)
                huge: 0.1,     // 10% huge (> 10MB)
            },
            ..Default::default()
        };
        
        let mut runner = SoakTestRunner::new(Duration::from_secs(12 * 3600), config);
        let results = runner.run().await.unwrap();
        
        // Memory should stabilize after initial growth
        let stable_region = &results.memory_samples[results.memory_samples.len() / 2..];
        let growth = calculate_memory_growth(stable_region);
        assert!(
            growth < 5.0,
            "Memory still growing after stabilization: {:.2} MB/hour",
            growth
        );
    }
}
```

### 6. Reporting
```rust
impl SoakTestResults {
    pub async fn generate_report(&self, path: &str) -> Result<(), Error> {
        use plotters::prelude::*;
        
        let root = BitMapBackend::new(path, (1920, 1080)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Soak Test Results", ("sans-serif", 50))
            .build_cartesian_2d(
                0f64..self.duration.as_secs_f64(),
                0f64..self.max_memory(),
            )?;
        
        // Plot memory usage over time
        chart.draw_series(LineSeries::new(
            self.memory_samples.iter().map(|s| {
                (s.timestamp.as_secs_f64(), s.heap_used as f64)
            }),
            &RED,
        ))?
        .label("Heap Usage")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &RED));
        
        // Plot latency percentiles
        chart.draw_series(LineSeries::new(
            self.latency_samples.iter().map(|s| {
                (s.timestamp.as_secs_f64(), s.p95)
            }),
            &BLUE,
        ))?
        .label("p95 Latency")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], &BLUE));
        
        root.present()?;
        
        // Generate HTML report with detailed analysis
        let html = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Soak Test Report</title>
                <style>
                    body {{ font-family: Arial, sans-serif; }}
                    .metric {{ padding: 10px; margin: 10px; border: 1px solid #ccc; }}
                    .pass {{ background-color: #d4edda; }}
                    .fail {{ background-color: #f8d7da; }}
                </style>
            </head>
            <body>
                <h1>Soak Test Results</h1>
                <div class="metric {}">
                    <h2>Overall Result: {}</h2>
                </div>
                <div class="metric">
                    <h3>Test Duration</h3>
                    <p>{} hours</p>
                </div>
                <div class="metric">
                    <h3>Total Requests</h3>
                    <p>{:,}</p>
                </div>
                <div class="metric">
                    <h3>Success Rate</h3>
                    <p>{:.2}%</p>
                </div>
                <div class="metric">
                    <h3>Memory Leaks</h3>
                    <p>{}</p>
                </div>
                <div class="metric">
                    <h3>Performance Anomalies</h3>
                    <p>{}</p>
                </div>
                <img src="{}" alt="Charts" />
            </body>
            </html>
            "#,
            if self.final_verdict == TestVerdict::Passed { "pass" } else { "fail" },
            self.final_verdict,
            self.duration.as_secs() / 3600,
            self.total_requests,
            self.successful_requests as f64 / self.total_requests as f64 * 100.0,
            self.resource_leaks.len(),
            self.performance_anomalies.len(),
            path
        );
        
        std::fs::write(path.replace(".png", ".html"), html)?;
        
        Ok(())
    }
}
```

## Implementation Steps

1. **Create soak test framework** (2 hours)
   - Test runner with configurable duration
   - Load generator with operation mix
   - Metrics collection

2. **Implement memory leak detection** (1.5 hours)
   - Heap growth analysis
   - Handle counting
   - Allocation profiling

3. **Add performance monitoring** (1 hour)
   - Latency tracking
   - Throughput monitoring
   - Degradation detection

4. **Platform-specific monitoring** (1 hour)
   - Linux: procfs integration
   - macOS: mach API integration
   - Windows: Performance counters

5. **Create test scenarios** (1 hour)
   - 24-hour moderate load
   - 48-hour with reconnects
   - Memory stress test

6. **Build reporting system** (30 min)
   - Charts generation
   - HTML report
   - CI integration

## Testing Strategy

1. **Incremental Testing**
   - Start with 1-hour tests
   - Progress to 12-hour tests
   - Full 24-48 hour validation

2. **Resource Monitoring**
   - Memory usage tracking
   - File descriptor counting
   - Thread count monitoring

3. **Baseline Establishment**
   - Run without load to establish baseline
   - Compare loaded vs unloaded growth

## Success Criteria

- [ ] No memory growth > 10MB/hour after stabilization
- [ ] No handle leaks (stable FD count)
- [ ] Performance degradation < 10% over 24 hours
- [ ] 99.9% uptime during soak test
- [ ] All resources properly cleaned up

## Risk Mitigation

1. **Test Environment**: Use dedicated hardware/VMs for soak tests
2. **Monitoring**: Set up alerts for resource exhaustion
3. **Automation**: Run soak tests in CI on schedule

## Dependencies
- Core MCP implementation
- Metrics system
- Platform-specific APIs

## Estimated Duration
7 hours

## Notes
- Consider using valgrind/AddressSanitizer for additional leak detection
- May need different thresholds for different deployment scenarios
- Document baseline resource usage for capacity planning