# Task B.2: Load Balancing Strategies Implementation

**Task ID**: B.2  
**Depends On**: B.1 (Multiple Upstream Support)  
**Estimated Duration**: 3 hours  
**Priority**: HIGH  
**Status**: Not Started

## Objective

Implement CLI support for all load balancing strategies available in the reverse proxy module, enabling users to distribute traffic across multiple upstream servers with various algorithms.

## Background

The reverse proxy module already implements multiple load balancing strategies in the `ReverseLoadBalancingStrategy` enum, but these are not accessible via CLI. This task will expose all strategies through command-line arguments and configuration files.

## Requirements

### Functional Requirements
1. Support all existing load balancing strategies via CLI
2. Allow strategy selection with `--load-balancing` flag
3. Support weighted strategies with upstream weight configuration
4. Provide sensible defaults for each strategy
5. Validate strategy compatibility with upstream configuration

### Strategies to Implement
- `round-robin` - Distribute requests evenly in order
- `weighted-round-robin` - Distribute based on weights
- `least-connections` - Route to server with fewest active connections
- `random` - Random selection
- `weighted-random` - Random selection based on weights
- `healthy-first` - Prefer healthy servers, fallback to others

## Implementation Plan

### Step 1: Extend CLI Arguments (30 min)
```rust
// In src/cli/reverse.rs
#[derive(Debug, Args)]
pub struct ReverseCommand {
    // ... existing fields ...
    
    /// Load balancing strategy for multiple upstreams
    #[arg(long, value_enum)]
    pub load_balancing: Option<LoadBalancingStrategy>,
    
    /// Enable sticky sessions (session affinity)
    #[arg(long)]
    pub sticky_sessions: bool,
    
    /// Cookie name for sticky sessions
    #[arg(long, default_value = "shadowcat_session")]
    pub session_cookie: String,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    WeightedRandom,
    HealthyFirst,
}
```

### Step 2: Parse Weighted Upstreams (45 min)
```rust
// Parse upstream format: "name=url,weight=N"
pub fn parse_upstream_spec(spec: &str) -> Result<ParsedUpstream> {
    let mut parts = spec.split(',');
    let main = parts.next().ok_or("Empty upstream spec")?;
    
    let (name, url) = if main.contains('=') {
        let mut kv = main.split('=');
        (Some(kv.next()?), kv.next()?)
    } else {
        (None, main)
    };
    
    let mut weight = 1u32;
    for part in parts {
        if let Some(w) = part.strip_prefix("weight=") {
            weight = w.parse()?;
        }
    }
    
    Ok(ParsedUpstream { name, url, weight })
}
```

### Step 3: Build Configuration (45 min)
```rust
impl ReverseCommand {
    fn build_load_balancing_config(&self) -> Result<ReverseProxyConfig> {
        let mut config = ReverseProxyConfig::default();
        
        // Parse all upstreams
        let upstreams: Vec<ReverseUpstreamConfig> = self.upstream
            .iter()
            .map(|spec| parse_upstream_spec(spec))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .enumerate()
            .map(|(i, parsed)| {
                ReverseUpstreamConfig {
                    id: parsed.name.unwrap_or_else(|| format!("upstream-{}", i)),
                    weight: parsed.weight,
                    // ... other fields
                }
            })
            .collect();
        
        // Validate strategy compatibility
        if upstreams.len() == 1 && self.load_balancing.is_some() {
            warn!("Load balancing specified but only one upstream configured");
        }
        
        // Set strategy
        config.load_balancing_strategy = match self.load_balancing {
            Some(s) => s.into(),
            None if upstreams.len() > 1 => ReverseLoadBalancingStrategy::RoundRobin,
            None => ReverseLoadBalancingStrategy::RoundRobin,
        };
        
        // Validate weighted strategies have weights
        if matches!(
            config.load_balancing_strategy,
            ReverseLoadBalancingStrategy::WeightedRoundRobin |
            ReverseLoadBalancingStrategy::WeightedRandom
        ) {
            if upstreams.iter().all(|u| u.weight == 1) {
                warn!("Weighted strategy selected but no weights specified, using equal weights");
            }
        }
        
        config.upstream_configs = upstreams;
        Ok(config)
    }
}
```

### Step 4: Add Connection Tracking (30 min)
For least-connections strategy:
```rust
// Add connection counter to upstream state
pub struct UpstreamState {
    pub id: String,
    pub active_connections: Arc<AtomicUsize>,
    pub total_requests: Arc<AtomicU64>,
    pub failed_requests: Arc<AtomicU64>,
    pub health_status: Arc<RwLock<HealthStatus>>,
}

// Update selection logic
impl LoadBalancer {
    pub fn select_upstream(&self) -> Option<&UpstreamState> {
        match self.strategy {
            LeastConnections => {
                self.upstreams
                    .iter()
                    .filter(|u| u.is_healthy())
                    .min_by_key(|u| u.active_connections.load(Ordering::Relaxed))
            }
            // ... other strategies
        }
    }
}
```

### Step 5: Implement Sticky Sessions (30 min)
```rust
pub struct SessionAffinity {
    cookie_name: String,
    session_map: Arc<RwLock<HashMap<String, String>>>,
}

impl SessionAffinity {
    pub fn get_upstream_for_session(&self, session_id: &str) -> Option<String> {
        self.session_map.read().await.get(session_id).cloned()
    }
    
    pub fn set_upstream_for_session(&self, session_id: String, upstream_id: String) {
        self.session_map.write().await.insert(session_id, upstream_id);
    }
}
```

### Step 6: Add Tests (30 min)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_weighted_upstream() {
        let spec = "primary=http://server1:8080,weight=3";
        let parsed = parse_upstream_spec(spec).unwrap();
        assert_eq!(parsed.name, Some("primary"));
        assert_eq!(parsed.weight, 3);
    }
    
    #[test]
    fn test_round_robin_selection() {
        let balancer = LoadBalancer::new(RoundRobin, upstreams);
        let selections: Vec<_> = (0..4).map(|_| balancer.select()).collect();
        assert_eq!(selections, vec!["upstream-0", "upstream-1", "upstream-0", "upstream-1"]);
    }
    
    #[test]
    fn test_weighted_distribution() {
        // Test that weighted selection respects ratios
    }
    
    #[test]
    fn test_least_connections() {
        // Test that least loaded server is selected
    }
}
```

## Validation Requirements

### Input Validation
- Strategy must be valid enum value
- Weights must be positive integers (1-100)
- Weighted strategies should have meaningful weights
- Multiple upstreams required for load balancing

### Configuration Validation
```rust
fn validate_load_balancing(config: &ReverseProxyConfig) -> Result<()> {
    // Single upstream with load balancing is warning, not error
    if config.upstream_configs.len() == 1 && 
       config.load_balancing_strategy != RoundRobin {
        warn!("Load balancing configured but only one upstream available");
    }
    
    // Weighted strategies need weights
    if matches!(config.load_balancing_strategy, WeightedRoundRobin | WeightedRandom) {
        let has_weights = config.upstream_configs.iter().any(|u| u.weight != 1);
        if !has_weights {
            info!("Using weighted strategy with equal weights");
        }
    }
    
    Ok(())
}
```

## Testing Checklist

- [ ] Unit tests for upstream parsing
- [ ] Unit tests for each strategy
- [ ] Integration test for multi-upstream with load balancing
- [ ] Test weight distribution accuracy
- [ ] Test sticky session persistence
- [ ] Test health-aware selection
- [ ] Test connection counting accuracy
- [ ] Benchmark strategy performance

## Documentation Updates

### CLI Help Text
```
--load-balancing <STRATEGY>
    Load balancing strategy for multiple upstreams
    
    Possible values:
    - round-robin:         Distribute requests evenly in sequence
    - weighted-round-robin: Distribute based on upstream weights
    - least-connections:   Route to server with fewest connections
    - random:             Random upstream selection
    - weighted-random:    Random selection based on weights
    - healthy-first:      Prefer healthy servers
    
    Default: round-robin (when multiple upstreams configured)
    
--sticky-sessions
    Enable session affinity (requests from same client go to same upstream)
    
--session-cookie <NAME>
    Cookie name for sticky sessions [default: shadowcat_session]
```

### Example Commands
```bash
# Basic round-robin
shadowcat reverse \
  --upstream http://server1:8080 \
  --upstream http://server2:8080

# Weighted load balancing
shadowcat reverse \
  --upstream "primary=http://server1:8080,weight=3" \
  --upstream "secondary=http://server2:8080,weight=1" \
  --load-balancing weighted-round-robin

# Least connections with sticky sessions
shadowcat reverse \
  --upstream http://server1:8080 \
  --upstream http://server2:8080 \
  --load-balancing least-connections \
  --sticky-sessions
```

## Success Criteria

1. All 6 load balancing strategies accessible via CLI
2. Weighted strategies properly distribute load
3. Least-connections accurately tracks connections
4. Sticky sessions maintain affinity
5. Health-aware strategies skip unhealthy upstreams
6. No performance regression
7. All tests passing
8. Documentation complete

## Error Handling

Common errors and messages:
```
Error: Load balancing strategy 'weighted-round-robin' requires upstream weights
Hint: Use format --upstream "name=url,weight=N" to specify weights

Warning: Load balancing configured but only one upstream available
Info: Load balancing will be inactive until multiple upstreams are configured

Error: Invalid weight '0' for upstream 'primary'
Hint: Weights must be positive integers between 1 and 100
```

## Dependencies

- B.1: Multiple upstream support must be implemented first
- Existing `ReverseLoadBalancingStrategy` enum
- Existing `ReverseProxyServer` load balancer logic

## Notes

- Consider adding metrics for load distribution monitoring
- Future: Add dynamic weight adjustment based on response times
- Future: Add circuit breaker integration with load balancing
- Consider adding health check integration for healthy-first strategy