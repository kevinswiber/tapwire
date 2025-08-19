# Risk Assessment and Mitigations

## High Risks

- Pool return not processed
  - Cause: Receiver guarded by mutex; select starvation.
  - Mitigation: Own receiver in maintenance task; absorb first interval tick; add test that asserts reuse.

- Subprocess reuse fails silently
  - Cause: `is_connected()` stays true after EOF; dead connections marked healthy.
  - Mitigation: Flip `connected=false` on EOF; optionally check child status; add health gate in return path.

- Deadlocks/Contention under load
  - Cause: Awaiting while holding `idle_connections` lock.
  - Mitigation: Refactor to release lock before awaits; stress test with concurrent returns/acquires.

## Medium Risks

- SSE reconnection storms
  - Cause: No backoff/jitter; synchronized reconnects.
  - Mitigation: Exponential backoff with full jitter; staggered initial delays.

- Performance regressions undetected
  - Cause: No standardized benchmark harness.
  - Mitigation: Add script and CI job; set acceptance thresholds per task.

## Low Risks

- Misaligned expectations for stdio pooling
  - Cause: Lack of documentation for single-shot CLIs.
  - Mitigation: Add explicit docs and config guidance.

