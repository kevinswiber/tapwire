### Interceptor chain performance (Delta: shadowcat-delta@b793fd1)

Findings

- Interceptor chain takes write lock for metrics on every message and during finalization
```369:377:shadowcat-delta/src/interceptor/engine.rs
{ let mut metrics = self.metrics.write().await; metrics.total_messages += 1; }
…
440:445:shadowcat-delta/src/interceptor/engine.rs
let mut metrics = self.metrics.write().await;
metrics.total_processing_time_ms += total_duration.as_millis() as u64;
metrics.avg_processing_time_ms = metrics.total_processing_time_ms as f64 / metrics.total_messages as f64;
metrics.record_action(&final_action);
```
- Per-interceptor logging of completion with Debug formatting of action
```396:403:shadowcat-delta/src/interceptor/engine.rs
debug!("Interceptor '{}' completed in {:?} with action: {:?}", interceptor.name(), interceptor_duration, action);
```
- `RuleBasedInterceptor` reads lock for evaluate, then may write metrics; evaluation timeout wraps future per call
```551:572:shadowcat-delta/src/interceptor/rules_interceptor.rs
let engine = self.rule_engine.read().await;
let evaluation_result = if self.config.evaluation_timeout > Duration::ZERO {
    tokio::time::timeout(self.config.evaluation_timeout, engine.evaluate(ctx)).await?
} else { engine.evaluate(ctx).await };
```
- Rule loading/watching uses filesystem IO and parsing on reload path; fine, but ensure not on hot-path.

Recommendations

- Metrics overhead
  - Use `parking_lot::RwLock` or atomics for hot counters (`total_messages`, action counts) to avoid async lock.
  - Accumulate per-task/thread local metrics and flush periodically.
- Logging cost
  - Downgrade per-interceptor debug or guard with sampling. Avoid `{:?}` of large actions.
- Engine evaluation
  - Precompile matchers; ensure JSON-path and string matchers avoid allocations; reuse scratch buffers.
  - If many interceptors, short-circuit earlier; consider priority buckets and early exit.
- Timeout wrapper
  - Avoid allocating a timeout future when `evaluation_timeout` is zero; current code already branches; keep it.
