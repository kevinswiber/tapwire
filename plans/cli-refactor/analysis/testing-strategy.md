# Testing Strategy

## Overview
This document defines the testing approach for the CLI refactoring project, ensuring that functionality is preserved throughout the migration process.

## Testing Principles

### 1. Test Before You Move
Never extract code without tests that verify current behavior

### 2. Test After You Move
Verify identical behavior after extraction

### 3. Test in Isolation
Each module should be independently testable

### 4. Test the Integration
Ensure modules work together correctly

## Test Categories

### 1. Baseline Tests (Before Refactoring)
Create comprehensive tests of current behavior to serve as regression tests.

#### Command Output Capture
```bash
# Create test script to capture current behavior
#!/bin/bash
# baseline_tests.sh

# Test forward stdio
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | \
  cargo run -- forward stdio -- echo > baseline/forward_stdio.txt 2>&1

# Test forward http
timeout 5 cargo run -- forward http --port 8080 --target http://localhost:3000 \
  > baseline/forward_http.txt 2>&1 || true

# Test reverse proxy
timeout 5 cargo run -- reverse --bind 127.0.0.1:8080 --upstream http://localhost:3000 \
  > baseline/reverse.txt 2>&1 || true

# Test recording
echo '{"jsonrpc":"2.0","method":"ping","id":1}' | \
  cargo run -- record stdio --output test.tape -- echo \
  > baseline/record_stdio.txt 2>&1

# Test replay
cargo run -- replay test.tape --port 8080 \
  > baseline/replay.txt 2>&1 &
REPLAY_PID=$!
sleep 2
curl -X POST http://localhost:8080 -d '{"jsonrpc":"2.0","method":"ping","id":1}' \
  > baseline/replay_response.txt 2>&1
kill $REPLAY_PID

# Test tape commands
cargo run -- tape list > baseline/tape_list.txt 2>&1
cargo run -- tape info test.tape > baseline/tape_info.txt 2>&1
```

#### Integration Test Suite
```rust
// tests/cli_integration_test.rs
#[test]
fn test_forward_stdio_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "forward", "stdio", "--", "echo"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    // Save output for comparison
}
```

### 2. Unit Tests (Per Module)

#### Common Module Tests
```rust
// src/cli/common.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_proxy_config_from_cli_args() {
        let config = ProxyConfig::from_cli_args(
            true, 100, 20, 300, 1000, 60
        );
        assert_eq!(config.enable_rate_limit, true);
        assert_eq!(config.rate_limit_rpm, 100);
    }
    
    #[tokio::test]
    async fn test_create_rate_limiter() {
        let config = ProxyConfig::from_cli_args(
            true, 100, 20, 300, 1000, 60
        );
        let limiter = config.create_rate_limiter().await.unwrap();
        assert!(limiter.is_some());
    }
    
    #[test]
    fn test_json_conversion() {
        let json = json!({"jsonrpc": "2.0", "method": "test", "id": 1});
        let msg = json_to_protocol_message(&json).unwrap();
        let back = protocol_message_to_json(&msg);
        assert_eq!(json, back);
    }
}
```

#### Command Module Tests
```rust
// src/cli/forward.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        Transport {}
        #[async_trait]
        impl Transport for Transport {
            async fn connect(&mut self) -> Result<()>;
            async fn send(&mut self, msg: MessageEnvelope) -> Result<()>;
            async fn receive(&mut self) -> Result<MessageEnvelope>;
            async fn close(&mut self) -> Result<()>;
        }
    }
    
    #[tokio::test]
    async fn test_forward_stdio_with_mock() {
        let mut mock_transport = MockTransport::new();
        mock_transport.expect_connect()
            .times(1)
            .returning(|| Ok(()));
        
        // Test forward logic with mock
    }
}
```

### 3. Integration Tests (End-to-End)

#### HTTP Server Tests
```rust
// tests/integration/http_forward_test.rs
use axum_test::TestServer;

#[tokio::test]
async fn test_http_forward_proxy() {
    // Start mock upstream server
    let upstream = TestServer::new(mock_upstream_app()).unwrap();
    
    // Start forward proxy
    let proxy = TestServer::new(
        forward::create_http_app(8080, upstream.url())
    ).unwrap();
    
    // Send request through proxy
    let response = proxy.post("/")
        .json(&json!({"jsonrpc": "2.0", "method": "test", "id": 1}))
        .await;
    
    assert_eq!(response.status(), 200);
}
```

#### Process Spawning Tests
```rust
#[tokio::test]
async fn test_stdio_command_spawning() {
    let args = StdioForwardArgs {
        command: vec!["echo".to_string(), "test".to_string()],
        config: ProxyConfig::default(),
    };
    
    let result = execute_stdio(args).await;
    assert!(result.is_ok());
}
```

### 4. Performance Tests

#### Benchmark Suite
```rust
// benches/cli_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_proxy_config(c: &mut Criterion) {
    c.bench_function("create_proxy_config", |b| {
        b.iter(|| {
            ProxyConfig::from_cli_args(
                black_box(true),
                black_box(100),
                black_box(20),
                black_box(300),
                black_box(1000),
                black_box(60),
            )
        });
    });
}

fn benchmark_rate_limiter(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("create_rate_limiter", |b| {
        b.to_async(&runtime).iter(|| async {
            let config = ProxyConfig::default();
            config.create_rate_limiter().await
        });
    });
}

criterion_group!(benches, benchmark_proxy_config, benchmark_rate_limiter);
criterion_main!(benches);
```

### 5. Regression Tests

#### Automated Comparison
```bash
#!/bin/bash
# regression_test.sh

# Run baseline capture
./baseline_tests.sh

# Run same tests after refactoring
./current_tests.sh

# Compare outputs
for file in baseline/*.txt; do
    basename=$(basename "$file")
    if ! diff -u "baseline/$basename" "current/$basename"; then
        echo "REGRESSION: $basename differs!"
        exit 1
    fi
done

echo "All regression tests passed!"
```

## Testing Tools

### Required Dependencies
```toml
[dev-dependencies]
mockall = "0.11"
tokio-test = "0.4"
axum-test = "0.5"
criterion = "0.5"
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
```

### Test Execution Commands
```bash
# Run all tests
cargo test --all

# Run specific module tests
cargo test cli::common
cargo test cli::forward

# Run integration tests
cargo test --test '*'

# Run benchmarks
cargo bench

# Run with coverage
cargo tarpaulin --out Html

# Run clippy
cargo clippy --all-targets -- -D warnings
```

## Test Coverage Goals

### Minimum Coverage Requirements
- Common module: 90%
- Command modules: 80%
- Handlers module: 85%
- Integration tests: Full command coverage

### Critical Path Coverage
These must have 100% test coverage:
- ProxyConfig creation and conversion
- Rate limiter initialization
- Session manager creation
- Error handling paths
- Command dispatch in main.rs

## Testing During Migration

### Phase 1: Foundation Testing
1. Write comprehensive tests for ProxyConfig
2. Test all utility functions
3. Verify session/rate limiter creation
4. Run baseline tests

### Phase 2: Simple Command Testing
1. Test replay command in isolation
2. Test record commands separately
3. Integration test with real tapes
4. Compare with baseline behavior

### Phase 3: Complex Command Testing
1. Mock transport for forward tests
2. Test HTTP server setup
3. Test command spawning
4. Full integration tests

### Phase 4: Final Validation
1. Complete regression test suite
2. Performance benchmarks
3. Binary size comparison
4. Load testing

## Continuous Testing

### Pre-commit Hooks
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Format check
cargo fmt -- --check || exit 1

# Clippy check
cargo clippy --all-targets -- -D warnings || exit 1

# Test check
cargo test --all || exit 1

echo "All pre-commit checks passed!"
```

### CI Pipeline
```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - name: Run tests
        run: cargo test --all
        
      - name: Run clippy
        run: cargo clippy --all-targets -- -D warnings
        
      - name: Check formatting
        run: cargo fmt -- --check
        
      - name: Run benchmarks
        run: cargo bench --no-run
```

## Test Documentation

### Test Naming Convention
- Unit tests: `test_<function_name>_<scenario>`
- Integration tests: `test_<feature>_<use_case>`
- Benchmarks: `benchmark_<operation>`

### Test Comments
```rust
/// Tests that ProxyConfig correctly converts CLI arguments
/// into a valid configuration structure with all fields set.
#[test]
fn test_proxy_config_from_cli_args_all_fields() {
    // Test implementation
}
```

## Rollback Testing

### Before Rollback
1. Save current test results
2. Document failure reason
3. Create minimal reproduction

### After Rollback
1. Verify tests pass again
2. Add regression test for issue
3. Document in migration notes

## Success Criteria

### All Tests Pass
- ✅ Unit tests: 100% pass rate
- ✅ Integration tests: 100% pass rate
- ✅ Regression tests: No differences
- ✅ Performance: No regression > 5%
- ✅ Clippy: No warnings

### Coverage Met
- ✅ Overall coverage > 80%
- ✅ Critical paths 100% covered
- ✅ New code > 90% covered

### Quality Gates
- ✅ No new compiler warnings
- ✅ No new clippy warnings
- ✅ Documentation complete
- ✅ Examples provided