# Task G.0: Fault Injection and Chaos Testing

## Objective
Implement comprehensive fault injection capabilities through a specialized interceptor and test harness to validate system resilience under adverse conditions.

## Background
From Gemini's review: "The plan focuses on 'happy path' and expected failure testing. It doesn't explicitly mention chaos engineering, fault injection, or testing for resource exhaustion scenarios."

## Key Requirements

### 1. FaultInjectorInterceptor
```rust
use std::sync::Arc;
use parking_lot::RwLock;
use rand::{Rng, thread_rng};

pub struct FaultInjectorInterceptor {
    config: Arc<RwLock<FaultConfig>>,
    rng: parking_lot::Mutex<rand::rngs::ThreadRng>,
}

#[derive(Debug, Clone)]
pub struct FaultConfig {
    /// Probability of injecting a fault (0.0 to 1.0)
    pub fault_probability: f64,
    
    /// Types of faults to inject
    pub enabled_faults: FaultTypes,
    
    /// Delay configuration
    pub delay_ms: DelayConfig,
    
    /// Error injection config
    pub error_config: ErrorConfig,
    
    /// Message corruption config
    pub corruption_config: CorruptionConfig,
    
    /// Resource exhaustion simulation
    pub resource_exhaustion: ResourceExhaustionConfig,
}

#[derive(Debug, Clone)]
pub struct FaultTypes {
    pub delay: bool,
    pub error: bool,
    pub corruption: bool,
    pub drop: bool,
    pub duplicate: bool,
    pub reorder: bool,
    pub resource_exhaustion: bool,
}

#[derive(Debug, Clone)]
pub struct DelayConfig {
    pub min_ms: u64,
    pub max_ms: u64,
    pub jitter: bool,
}

#[derive(Debug, Clone)]
pub struct ErrorConfig {
    pub types: Vec<ErrorType>,
    pub custom_messages: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Timeout,
    ConnectionRefused,
    InvalidData,
    InternalError,
    RateLimited,
    Unauthorized,
}

impl FaultInjectorInterceptor {
    pub fn new(config: FaultConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            rng: parking_lot::Mutex::new(thread_rng()),
        }
    }
    
    pub fn update_config(&self, config: FaultConfig) {
        *self.config.write() = config;
    }
    
    fn should_inject_fault(&self) -> bool {
        let config = self.config.read();
        let mut rng = self.rng.lock();
        rng.gen_bool(config.fault_probability)
    }
    
    fn select_fault(&self) -> Option<FaultAction> {
        if !self.should_inject_fault() {
            return None;
        }
        
        let config = self.config.read();
        let faults = &config.enabled_faults;
        let mut rng = self.rng.lock();
        
        // Weighted selection of fault types
        let enabled: Vec<(&str, bool)> = vec![
            ("delay", faults.delay),
            ("error", faults.error),
            ("corruption", faults.corruption),
            ("drop", faults.drop),
            ("duplicate", faults.duplicate),
            ("reorder", faults.reorder),
            ("resource_exhaustion", faults.resource_exhaustion),
        ];
        
        let active_faults: Vec<&str> = enabled
            .into_iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| name)
            .collect();
        
        if active_faults.is_empty() {
            return None;
        }
        
        let selected = active_faults[rng.gen_range(0..active_faults.len())];
        
        match selected {
            "delay" => {
                let delay_ms = rng.gen_range(config.delay_ms.min_ms..=config.delay_ms.max_ms);
                Some(FaultAction::Delay(Duration::from_millis(delay_ms)))
            },
            "error" => {
                let error_type = &config.error_config.types[
                    rng.gen_range(0..config.error_config.types.len())
                ];
                Some(FaultAction::Error(error_type.clone()))
            },
            "corruption" => Some(FaultAction::Corrupt),
            "drop" => Some(FaultAction::Drop),
            "duplicate" => Some(FaultAction::Duplicate),
            "reorder" => Some(FaultAction::Reorder),
            "resource_exhaustion" => Some(FaultAction::ResourceExhaustion),
            _ => None,
        }
    }
}

#[async_trait]
impl Interceptor for FaultInjectorInterceptor {
    async fn intercept_request(
        &self,
        request: &mut JsonRpcRequest,
        context: &InterceptorContext,
    ) -> Result<InterceptAction, InterceptorError> {
        match self.select_fault() {
            Some(FaultAction::Delay(duration)) => {
                tracing::debug!(?duration, "Injecting delay fault");
                tokio::time::sleep(duration).await;
                Ok(InterceptAction::Continue)
            },
            Some(FaultAction::Drop) => {
                tracing::debug!("Dropping request");
                Ok(InterceptAction::Block("Fault injection: dropped".into()))
            },
            Some(FaultAction::Corrupt) => {
                tracing::debug!("Corrupting request");
                self.corrupt_message(request);
                Ok(InterceptAction::Continue)
            },
            Some(FaultAction::Error(error_type)) => {
                tracing::debug!(?error_type, "Injecting error");
                Err(self.create_error(error_type))
            },
            Some(FaultAction::Duplicate) => {
                tracing::debug!("Duplicating request");
                // Store for later duplicate injection
                context.set_metadata("fault_duplicate", request.clone())?;
                Ok(InterceptAction::Continue)
            },
            Some(FaultAction::ResourceExhaustion) => {
                self.simulate_resource_exhaustion().await;
                Ok(InterceptAction::Continue)
            },
            _ => Ok(InterceptAction::Continue),
        }
    }
    
    fn corrupt_message(&self, request: &mut JsonRpcRequest) {
        let mut rng = self.rng.lock();
        match rng.gen_range(0..4) {
            0 => {
                // Corrupt method name
                request.method.push_str("_corrupted");
            },
            1 => {
                // Invalid JSON-RPC version
                request.jsonrpc = "1.0".to_string();
            },
            2 => {
                // Null params when params expected
                request.params = Some(serde_json::Value::Null);
            },
            _ => {
                // Corrupt ID
                request.id = Some(JsonRpcId::String("corrupted_id".into()));
            }
        }
    }
    
    async fn simulate_resource_exhaustion(&self) {
        let config = self.config.read();
        let exhaustion = &config.resource_exhaustion;
        
        if exhaustion.memory_spike {
            // Allocate temporary memory
            let _spike: Vec<u8> = vec![0; exhaustion.memory_spike_mb * 1024 * 1024];
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        if exhaustion.cpu_spike {
            // CPU intensive operation
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_millis(exhaustion.cpu_spike_ms) {
                // Busy loop
                std::hint::black_box(fibonacci(40));
            }
        }
    }
}

enum FaultAction {
    Delay(Duration),
    Error(ErrorType),
    Corrupt,
    Drop,
    Duplicate,
    Reorder,
    ResourceExhaustion,
}
```

### 2. Chaos Test Scenarios
```rust
#[cfg(test)]
mod chaos_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_cascading_failures() {
        let fault_injector = FaultInjectorInterceptor::new(FaultConfig {
            fault_probability: 0.3,
            enabled_faults: FaultTypes {
                delay: true,
                error: true,
                ..Default::default()
            },
            delay_ms: DelayConfig {
                min_ms: 100,
                max_ms: 5000,
                jitter: true,
            },
            ..Default::default()
        });
        
        let server = Server::builder()
            .bind("127.0.0.1:0")
            .interceptor(fault_injector.clone())
            .handler(test_handler)
            .build()
            .await
            .unwrap();
        
        // Run multiple clients concurrently
        let mut handles = vec![];
        for i in 0..100 {
            let client = create_client(server.local_addr());
            handles.push(tokio::spawn(async move {
                for j in 0..100 {
                    let result = client.request("test.method", json!({})).await;
                    // Track success/failure rates
                }
            }));
        }
        
        // Gradually increase fault probability
        for i in 1..=10 {
            tokio::time::sleep(Duration::from_secs(1)).await;
            fault_injector.update_config(FaultConfig {
                fault_probability: i as f64 / 10.0,
                ..fault_injector.config.read().clone()
            });
        }
        
        // Collect results and verify graceful degradation
        let results = futures::future::join_all(handles).await;
        assert_degradation_is_graceful(results);
    }
    
    #[tokio::test]
    async fn test_connection_storms() {
        let server = create_test_server().await;
        
        // Simulate connection storm
        let mut connections = vec![];
        for _ in 0..10000 {
            match TcpStream::connect(server.local_addr()).await {
                Ok(conn) => connections.push(conn),
                Err(_) => break, // Hit connection limit
            }
        }
        
        // Verify server still responsive
        let client = create_client(server.local_addr());
        let result = client.request("ping", json!({})).await;
        assert!(result.is_ok());
        
        // Verify proper connection limiting
        assert!(connections.len() <= MAX_CONNECTIONS);
    }
    
    #[tokio::test]
    async fn test_memory_pressure() {
        let server = create_test_server().await;
        
        // Create many sessions to trigger memory pressure
        let mut clients = vec![];
        for _ in 0..5000 {
            let client = create_client(server.local_addr());
            // Each client maintains a session
            client.initialize().await.unwrap();
            clients.push(client);
        }
        
        // Wait for cleanup to trigger
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Verify LRU eviction occurred
        let metrics = get_metrics();
        assert!(metrics.sessions_evicted > 0);
        
        // Verify server still functional
        let test_client = create_client(server.local_addr());
        assert!(test_client.request("ping", json!({})).await.is_ok());
    }
}
```

### 3. Network Condition Simulation
```rust
pub struct NetworkSimulator {
    latency: LatencySimulation,
    packet_loss: PacketLossSimulation,
    bandwidth: BandwidthSimulation,
}

impl NetworkSimulator {
    pub async fn apply_to_connection(&self, conn: &mut Connection) {
        // Add latency
        if let Some(delay) = self.latency.calculate_delay() {
            tokio::time::sleep(delay).await;
        }
        
        // Simulate packet loss
        if self.packet_loss.should_drop() {
            conn.reset().await;
            return;
        }
        
        // Throttle bandwidth
        if let Some(throttle_delay) = self.bandwidth.calculate_throttle(conn.pending_bytes()) {
            tokio::time::sleep(throttle_delay).await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct LatencySimulation {
    pub base_ms: u64,
    pub jitter_ms: u64,
    pub spike_probability: f64,
    pub spike_multiplier: f64,
}

impl LatencySimulation {
    pub fn calculate_delay(&self) -> Option<Duration> {
        let mut rng = thread_rng();
        let base = self.base_ms as f64;
        let jitter = rng.gen_range(0.0..self.jitter_ms as f64);
        
        let delay = if rng.gen_bool(self.spike_probability) {
            (base + jitter) * self.spike_multiplier
        } else {
            base + jitter
        };
        
        Some(Duration::from_millis(delay as u64))
    }
}
```

### 4. Chaos Test Harness
```rust
pub struct ChaosTestHarness {
    server: Server,
    clients: Vec<Client>,
    fault_injector: Arc<FaultInjectorInterceptor>,
    network_simulator: Arc<NetworkSimulator>,
    metrics_collector: MetricsCollector,
}

impl ChaosTestHarness {
    pub async fn run_scenario(&mut self, scenario: ChaosScenario) -> TestResults {
        match scenario {
            ChaosScenario::GradualDegradation => {
                self.test_gradual_degradation().await
            },
            ChaosScenario::SuddenFailure => {
                self.test_sudden_failure().await
            },
            ChaosScenario::IntermittentFailures => {
                self.test_intermittent_failures().await
            },
            ChaosScenario::CascadingFailure => {
                self.test_cascading_failure().await
            },
        }
    }
    
    async fn test_gradual_degradation(&mut self) -> TestResults {
        let mut results = TestResults::new();
        
        // Start with no faults
        self.fault_injector.update_config(FaultConfig {
            fault_probability: 0.0,
            ..Default::default()
        });
        
        // Gradually increase fault probability
        for i in 0..=100 {
            let probability = i as f64 / 100.0;
            self.fault_injector.update_config(FaultConfig {
                fault_probability: probability,
                enabled_faults: FaultTypes {
                    delay: true,
                    error: true,
                    drop: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            
            // Run load test at this fault level
            let metrics = self.run_load_test(Duration::from_secs(10)).await;
            results.record_metrics(probability, metrics);
            
            // Check if system collapsed
            if metrics.success_rate < 0.1 {
                results.collapse_point = Some(probability);
                break;
            }
        }
        
        results
    }
}
```

## Implementation Steps

1. **Create FaultInjectorInterceptor** (2 hours)
   - Implement all fault types
   - Configurable probability and parameters
   - Runtime configuration updates

2. **Build network simulator** (1.5 hours)
   - Latency injection with jitter
   - Packet loss simulation
   - Bandwidth throttling

3. **Implement chaos scenarios** (2 hours)
   - Gradual degradation
   - Sudden failures
   - Cascading failures
   - Resource exhaustion

4. **Create test harness** (1 hour)
   - Scenario runner
   - Metrics collection
   - Result analysis

5. **Write chaos tests** (1.5 hours)
   - Connection storms
   - Memory pressure
   - CPU exhaustion
   - Network partitions

6. **Document findings** (30 min)
   - Failure modes discovered
   - System limits identified
   - Recommended mitigations

## Testing Strategy

1. **Controlled Chaos**
   - Start with low fault probability
   - Gradually increase to find breaking points
   - Identify graceful vs catastrophic failures

2. **Scenario Testing**
   - Upstream service failure
   - Network partition
   - Resource exhaustion
   - Thundering herd

3. **Recovery Testing**
   - Remove faults and verify recovery
   - Check for resource leaks
   - Validate metric accuracy

## Success Criteria

- [ ] All fault types implemented
- [ ] System degrades gracefully under faults
- [ ] No panics under any fault condition
- [ ] Recovery is automatic when faults clear
- [ ] Performance metrics accurate under chaos

## Risk Mitigation

1. **Test Environment Isolation**: Run chaos tests in isolated environment
2. **Circuit Breakers**: Implement to prevent cascading failures
3. **Resource Limits**: Set hard limits to prevent exhaustion

## Dependencies
- Core MCP implementation complete
- Interceptor chain implemented
- Metrics system in place

## Estimated Duration
8.5 hours

## Notes
- Consider using `tokio-test` for time manipulation in tests
- May want to integrate with existing chaos engineering tools
- Document all discovered failure modes for operational runbook