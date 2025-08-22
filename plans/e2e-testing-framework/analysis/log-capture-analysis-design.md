# Log Capture and Analysis Framework Design

## Overview
Comprehensive log capture is essential for debugging E2E test failures, especially in CI environments where reproduction can be difficult. The framework must capture logs from multiple processes, provide real-time analysis, and enable post-mortem debugging.

## Core Requirements

1. **Multi-Process Capture**: Collect logs from proxy, servers, and clients
2. **Structured Logging**: Parse and categorize log entries
3. **Real-Time Analysis**: Detect errors and anomalies during execution
4. **Pattern Matching**: Assert on specific log patterns
5. **Performance Metrics**: Extract timing and throughput data
6. **Failure Diagnostics**: Automatic log extraction on test failure
7. **CI Integration**: Archive logs for failed tests

## Architecture

### 1. Log Collection Pipeline

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use chrono::{DateTime, Utc};

pub struct LogCollector {
    // Channel for aggregating logs from multiple sources
    aggregator: mpsc::Sender<LogEntry>,
    
    // Structured storage
    storage: Arc<RwLock<LogStorage>>,
    
    // Real-time analyzers
    analyzers: Vec<Box<dyn LogAnalyzer>>,
    
    // Configuration
    config: LogConfig,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub source: LogSource,
    pub level: LogLevel,
    pub module: Option<String>,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub enum LogSource {
    Process { name: String, pid: u32 },
    Stdout,
    Stderr,
    File { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
```

### 2. Log Parsing and Structuring

```rust
pub trait LogParser: Send + Sync {
    fn parse(&self, line: &str) -> Option<LogEntry>;
}

pub struct TracingLogParser;

impl LogParser for TracingLogParser {
    fn parse(&self, line: &str) -> Option<LogEntry> {
        // Parse tracing/env_logger format
        // Example: "2024-01-15T10:30:45.123Z ERROR shadowcat::proxy: Connection failed"
        
        let re = regex::Regex::new(
            r"^(\S+)\s+(\w+)\s+([^:]+):\s+(.*)$"
        ).ok()?;
        
        let caps = re.captures(line)?;
        
        Some(LogEntry {
            timestamp: DateTime::parse_from_rfc3339(&caps[1])
                .ok()?
                .with_timezone(&Utc),
            level: parse_level(&caps[2]),
            module: Some(caps[3].to_string()),
            message: caps[4].to_string(),
            fields: extract_fields(&caps[4]),
            raw: line.to_string(),
            source: LogSource::Stdout,
        })
    }
}

pub struct JsonLogParser;

impl LogParser for JsonLogParser {
    fn parse(&self, line: &str) -> Option<LogEntry> {
        let json: serde_json::Value = serde_json::from_str(line).ok()?;
        
        Some(LogEntry {
            timestamp: json["timestamp"]
                .as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            level: json["level"]
                .as_str()
                .map(parse_level)
                .unwrap_or(LogLevel::Info),
            module: json["module"].as_str().map(String::from),
            message: json["message"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            fields: json["fields"]
                .as_object()
                .cloned()
                .unwrap_or_default(),
            raw: line.to_string(),
            source: LogSource::Stdout,
        })
    }
}
```

### 3. Real-Time Stream Processing

```rust
impl LogCollector {
    pub async fn capture_stream(
        &self,
        reader: impl AsyncBufRead + Unpin,
        source: LogSource,
        parser: Box<dyn LogParser>,
    ) {
        let mut lines = BufReader::new(reader).lines();
        let sender = self.aggregator.clone();
        
        while let Some(line) = lines.next_line().await.ok().flatten() {
            // Parse the log line
            if let Some(mut entry) = parser.parse(&line) {
                entry.source = source.clone();
                
                // Send to aggregator
                let _ = sender.send(entry.clone()).await;
                
                // Run real-time analyzers
                for analyzer in &self.analyzers {
                    analyzer.analyze(&entry).await;
                }
            }
        }
    }
    
    pub async fn start_aggregator(self: Arc<Self>) {
        let mut receiver = self.aggregator.subscribe();
        
        while let Some(entry) = receiver.recv().await {
            // Store the entry
            self.storage.write().await.add(entry.clone());
            
            // Check for critical patterns
            if entry.level >= LogLevel::Error {
                self.handle_error_log(&entry).await;
            }
        }
    }
}
```

### 4. Pattern Matching and Assertions

```rust
pub struct LogAssertions {
    storage: Arc<RwLock<LogStorage>>,
}

impl LogAssertions {
    pub async fn assert_pattern(&self, pattern: &str) -> Result<()> {
        let storage = self.storage.read().await;
        let re = regex::Regex::new(pattern)?;
        
        let found = storage.entries.iter().any(|entry| {
            re.is_match(&entry.message) || re.is_match(&entry.raw)
        });
        
        if !found {
            return Err(anyhow!(
                "Pattern '{}' not found in {} log entries",
                pattern,
                storage.entries.len()
            ));
        }
        
        Ok(())
    }
    
    pub async fn assert_no_errors(&self) -> Result<()> {
        let storage = self.storage.read().await;
        
        let errors: Vec<_> = storage.entries
            .iter()
            .filter(|e| e.level >= LogLevel::Error)
            .collect();
        
        if !errors.is_empty() {
            let error_messages: Vec<_> = errors
                .iter()
                .map(|e| format!("{}: {}", e.module.as_deref().unwrap_or("unknown"), e.message))
                .collect();
            
            return Err(anyhow!(
                "Found {} errors in logs:\n{}",
                errors.len(),
                error_messages.join("\n")
            ));
        }
        
        Ok(())
    }
    
    pub async fn assert_sequence(&self, patterns: &[&str]) -> Result<()> {
        let storage = self.storage.read().await;
        let mut pattern_idx = 0;
        
        for entry in &storage.entries {
            if pattern_idx >= patterns.len() {
                break;
            }
            
            let re = regex::Regex::new(patterns[pattern_idx])?;
            if re.is_match(&entry.message) {
                pattern_idx += 1;
            }
        }
        
        if pattern_idx < patterns.len() {
            return Err(anyhow!(
                "Sequence not found. Got to pattern {} of {}",
                pattern_idx,
                patterns.len()
            ));
        }
        
        Ok(())
    }
    
    pub async fn wait_for_pattern(
        &self,
        pattern: &str,
        timeout: Duration,
    ) -> Result<LogEntry> {
        let deadline = Instant::now() + timeout;
        let re = regex::Regex::new(pattern)?;
        
        while Instant::now() < deadline {
            let storage = self.storage.read().await;
            
            if let Some(entry) = storage.entries.iter().find(|e| {
                re.is_match(&e.message) || re.is_match(&e.raw)
            }) {
                return Ok(entry.clone());
            }
            
            drop(storage);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Err(anyhow!("Pattern '{}' not found within timeout", pattern))
    }
}
```

### 5. Performance Metrics Extraction

```rust
pub struct MetricsExtractor {
    patterns: HashMap<String, MetricPattern>,
}

#[derive(Debug, Clone)]
pub struct MetricPattern {
    pub name: String,
    pub regex: regex::Regex,
    pub value_group: usize,
    pub unit: MetricUnit,
}

#[derive(Debug, Clone)]
pub enum MetricUnit {
    Milliseconds,
    Microseconds,
    Bytes,
    Count,
    Percentage,
}

impl MetricsExtractor {
    pub fn extract_from_logs(&self, entries: &[LogEntry]) -> HashMap<String, Vec<f64>> {
        let mut metrics: HashMap<String, Vec<f64>> = HashMap::new();
        
        for entry in entries {
            for (name, pattern) in &self.patterns {
                if let Some(captures) = pattern.regex.captures(&entry.message) {
                    if let Some(value_str) = captures.get(pattern.value_group) {
                        if let Ok(value) = value_str.as_str().parse::<f64>() {
                            metrics.entry(name.clone())
                                .or_default()
                                .push(value);
                        }
                    }
                }
            }
        }
        
        metrics
    }
    
    pub fn calculate_statistics(&self, values: &[f64]) -> MetricStats {
        if values.is_empty() {
            return MetricStats::default();
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        MetricStats {
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            mean: values.iter().sum::<f64>() / values.len() as f64,
            median: sorted[sorted.len() / 2],
            p95: sorted[(sorted.len() as f64 * 0.95) as usize],
            p99: sorted[(sorted.len() as f64 * 0.99) as usize],
        }
    }
}
```

### 6. Log Storage and Rotation

```rust
pub struct LogStorage {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    total_size: usize,
    max_size: usize,
}

impl LogStorage {
    pub fn add(&mut self, entry: LogEntry) {
        let entry_size = entry.raw.len();
        
        // Rotate if necessary
        while self.entries.len() >= self.max_entries || 
              self.total_size + entry_size > self.max_size {
            if let Some(old) = self.entries.pop_front() {
                self.total_size -= old.raw.len();
            }
        }
        
        self.total_size += entry_size;
        self.entries.push_back(entry);
    }
    
    pub fn query(&self, filter: LogFilter) -> Vec<&LogEntry> {
        self.entries
            .iter()
            .filter(|e| filter.matches(e))
            .collect()
    }
    
    pub async fn save_to_file(&self, path: &Path) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        
        let mut file = tokio::fs::File::create(path).await?;
        
        for entry in &self.entries {
            file.write_all(entry.raw.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }
        
        file.flush().await?;
        Ok(())
    }
}
```

### 7. Error Detection and Alerting

```rust
#[async_trait]
pub trait LogAnalyzer: Send + Sync {
    async fn analyze(&self, entry: &LogEntry);
}

pub struct ErrorDetector {
    error_patterns: Vec<regex::Regex>,
    alerts: Arc<RwLock<Vec<Alert>>>,
}

#[async_trait]
impl LogAnalyzer for ErrorDetector {
    async fn analyze(&self, entry: &LogEntry) {
        // Check log level
        if entry.level >= LogLevel::Error {
            self.create_alert(
                AlertLevel::Error,
                format!("Error log detected: {}", entry.message),
                entry.clone(),
            ).await;
        }
        
        // Check patterns
        for pattern in &self.error_patterns {
            if pattern.is_match(&entry.message) {
                self.create_alert(
                    AlertLevel::Warning,
                    format!("Error pattern detected: {}", pattern.as_str()),
                    entry.clone(),
                ).await;
            }
        }
    }
}

pub struct PerformanceMonitor {
    slow_thresholds: HashMap<String, Duration>,
}

#[async_trait]
impl LogAnalyzer for PerformanceMonitor {
    async fn analyze(&self, entry: &LogEntry) {
        // Extract timing information
        if let Some(duration) = extract_duration(&entry.message) {
            if let Some(operation) = extract_operation(&entry.message) {
                if let Some(threshold) = self.slow_thresholds.get(&operation) {
                    if duration > *threshold {
                        warn!(
                            "Slow operation detected: {} took {:?} (threshold: {:?})",
                            operation, duration, threshold
                        );
                    }
                }
            }
        }
    }
}
```

### 8. Test Integration

```rust
pub struct TestLogger {
    collector: Arc<LogCollector>,
    assertions: LogAssertions,
    metrics: MetricsExtractor,
}

impl TestLogger {
    pub async fn capture_process(
        &self,
        child: &mut tokio::process::Child,
        name: &str,
    ) -> Result<()> {
        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("No stdout"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| anyhow!("No stderr"))?;
        
        let source = LogSource::Process {
            name: name.to_string(),
            pid: child.id().unwrap_or(0),
        };
        
        // Capture both streams
        tokio::spawn(self.collector.clone().capture_stream(
            BufReader::new(stdout),
            source.clone(),
            Box::new(TracingLogParser),
        ));
        
        tokio::spawn(self.collector.clone().capture_stream(
            BufReader::new(stderr),
            source,
            Box::new(TracingLogParser),
        ));
        
        Ok(())
    }
    
    pub async fn assert_healthy(&self) -> Result<()> {
        self.assertions.assert_no_errors().await?;
        self.assertions.assert_pattern(r"started successfully").await?;
        Ok(())
    }
    
    pub async fn extract_metrics(&self) -> HashMap<String, MetricStats> {
        let storage = self.collector.storage.read().await;
        let raw_metrics = self.metrics.extract_from_logs(&storage.entries);
        
        raw_metrics
            .into_iter()
            .map(|(name, values)| {
                (name, self.metrics.calculate_statistics(&values))
            })
            .collect()
    }
}
```

### 9. CI/CD Integration

```rust
pub struct CILogHandler {
    test_name: String,
    artifacts_dir: PathBuf,
}

impl CILogHandler {
    pub async fn on_test_failure(&self, logger: &TestLogger) -> Result<()> {
        // Create test-specific directory
        let test_dir = self.artifacts_dir.join(&self.test_name);
        tokio::fs::create_dir_all(&test_dir).await?;
        
        // Save all logs
        let storage = logger.collector.storage.read().await;
        storage.save_to_file(&test_dir.join("full.log")).await?;
        
        // Save error logs separately
        let errors: Vec<_> = storage.entries
            .iter()
            .filter(|e| e.level >= LogLevel::Error)
            .cloned()
            .collect();
        
        if !errors.is_empty() {
            let error_file = test_dir.join("errors.log");
            let mut file = tokio::fs::File::create(error_file).await?;
            
            for entry in errors {
                use tokio::io::AsyncWriteExt;
                file.write_all(format!("{}\n", entry.raw).as_bytes()).await?;
            }
        }
        
        // Extract and save metrics
        let metrics = logger.extract_metrics().await;
        let metrics_json = serde_json::to_string_pretty(&metrics)?;
        tokio::fs::write(
            test_dir.join("metrics.json"),
            metrics_json
        ).await?;
        
        // Create summary report
        self.create_summary_report(&test_dir, &storage).await?;
        
        Ok(())
    }
    
    async fn create_summary_report(
        &self,
        dir: &Path,
        storage: &LogStorage,
    ) -> Result<()> {
        let mut report = String::new();
        
        report.push_str(&format!("Test: {}\n", self.test_name));
        report.push_str(&format!("Total logs: {}\n", storage.entries.len()));
        
        // Count by level
        let mut level_counts: HashMap<LogLevel, usize> = HashMap::new();
        for entry in &storage.entries {
            *level_counts.entry(entry.level.clone()).or_default() += 1;
        }
        
        report.push_str("\nLog levels:\n");
        for (level, count) in level_counts {
            report.push_str(&format!("  {:?}: {}\n", level, count));
        }
        
        // Last 10 errors
        let errors: Vec<_> = storage.entries
            .iter()
            .filter(|e| e.level >= LogLevel::Error)
            .take(10)
            .collect();
        
        if !errors.is_empty() {
            report.push_str("\nRecent errors:\n");
            for error in errors {
                report.push_str(&format!("  {}\n", error.message));
            }
        }
        
        tokio::fs::write(dir.join("summary.txt"), report).await?;
        Ok(())
    }
}
```

## Usage Examples

### 1. Basic Test with Log Assertions

```rust
#[tokio::test]
async fn test_proxy_startup() {
    let logger = TestLogger::new();
    
    // Start proxy and capture logs
    let mut proxy = start_shadowcat_proxy().await?;
    logger.capture_process(&mut proxy, "shadowcat").await?;
    
    // Wait for startup
    logger.assertions
        .wait_for_pattern(r"Proxy listening on", Duration::from_secs(5))
        .await?;
    
    // Assert no errors during startup
    logger.assertions.assert_no_errors().await?;
    
    // Check expected log sequence
    logger.assertions.assert_sequence(&[
        r"Loading configuration",
        r"Initializing session manager",
        r"Starting HTTP server",
        r"Proxy listening on",
    ]).await?;
}
```

### 2. Performance Testing with Metrics

```rust
#[tokio::test]
async fn test_proxy_performance() {
    let logger = TestLogger::new()
        .with_metric("request_time", r"Request completed in (\d+)ms", 1, MetricUnit::Milliseconds)
        .with_metric("throughput", r"Throughput: (\d+) req/s", 1, MetricUnit::Count);
    
    // Run load test
    run_load_test(&logger).await?;
    
    // Extract and validate metrics
    let metrics = logger.extract_metrics().await;
    let request_time = &metrics["request_time"];
    
    assert!(request_time.p95 < 100.0, "P95 latency too high: {:.2}ms", request_time.p95);
    assert!(request_time.p99 < 200.0, "P99 latency too high: {:.2}ms", request_time.p99);
}
```

## Summary

This log framework provides:
- **Comprehensive Capture**: All process outputs collected
- **Structured Analysis**: Parse and categorize logs
- **Real-Time Detection**: Immediate error alerting
- **Pattern Assertions**: Flexible log validation
- **Performance Metrics**: Extract timing data
- **CI Integration**: Automatic artifact collection
- **Debugging Support**: Rich failure diagnostics