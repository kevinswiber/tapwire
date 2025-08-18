# Phase H: Critical Fixes from Review

This directory contains detailed task files for fixing all critical issues identified in the comprehensive review of the refactor/legacy-reverse-proxy branch.

## Task Organization

Tasks are numbered by priority and estimated duration:

### 🔴 Critical Issues (MUST FIX before merge)
- **H.0** - Fix Connection Pool Leak (2h)
- **H.1** - Fix Stdio Subprocess Spawning (4h)  
- **H.2** - Add Server Drop Implementation (2h)
- **H.3** - Deduplicate AppState Creation (1h)
- **H.4** - Implement SSE Reconnection (6h)

### 🟡 High Priority Issues
- **H.5** - Add Request Timeouts (3h)
- **H.6** - Restore Buffer Pooling (2h)
- **H.7** - Restore Admin Endpoints (4h)
- **H.8** - Restore Rate Limiting Tests (2h)
- **H.9** - Performance Benchmarks (3h)

### 🟢 Medium Priority
- **H.10** - Migration Documentation (2h)

## Execution Order

Recommended order for maximum efficiency:

**Day 1 (8 hours)**
1. H.0 - Fix connection pool leak (2h)
2. H.1 - Fix stdio spawning (4h)
3. H.2 - Add Drop implementation (2h)

**Day 2 (8 hours)**
1. H.3 - Deduplicate AppState (1h)
2. H.4 - Implement SSE reconnection (6h)
3. H.5 - Add request timeouts (1h of 3h)

**Day 3 (8 hours)**
1. H.5 - Complete timeouts (2h remaining)
2. H.6 - Restore buffer pooling (2h)
3. H.8 - Restore tests (2h)
4. H.9 - Performance benchmarks (2h)

**Day 4 (if needed)**
1. H.7 - Restore admin endpoints or document (4h)
2. H.10 - Migration documentation (2h)

## Success Criteria

Before marking Phase H complete, ALL of the following must be true:

### Resource Management
- [ ] No connection pool leaks under any condition
- [ ] All spawned tasks tracked and cleaned up
- [ ] Proper Drop implementation for all resources
- [ ] Memory usage stable under sustained load

### Performance
- [ ] P95 latency regression < 5% vs legacy
- [ ] Memory usage increase < 10% vs legacy
- [ ] Throughput reduction < 5% vs legacy
- [ ] Stdio requests complete in < 20ms

### Functionality
- [ ] SSE connections reconnect automatically
- [ ] Request timeouts prevent hanging
- [ ] All critical tests restored
- [ ] Breaking changes documented

### Testing
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Load test: 1000 connections for 1 hour
- [ ] Memory leak test: valgrind clean
- [ ] Performance benchmarks meet targets

## Quick Validation Commands

```bash
# Check for resource leaks
valgrind --leak-check=full ./target/release/shadowcat

# Performance benchmark
cargo bench --bench reverse_proxy

# Load test
./scripts/load-test.sh --connections 1000 --duration 3600

# Full test suite
cargo test --all-features --release

# Clippy check
cargo clippy --all-targets -- -D warnings
```

## Review Documents

For full context, see `/plans/refactor-legacy-reverse-proxy/reviews/`:
- `01-executive-summary.md` - High-level findings
- `02-technical-analysis.md` - Detailed code analysis
- `03-resource-performance-analysis.md` - Performance metrics
- `04-recommendations-action-items.md` - Prioritized fixes
- `05-critical-issues-checklist.md` - Quick reference

## Notes

- Each task file contains detailed implementation steps
- Code examples are provided for clarity
- Testing requirements are specified
- Dependencies between tasks are noted
- Alternative approaches are documented where applicable