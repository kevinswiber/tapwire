# SSE Reconnection Considerations

## Policy

- Backoff: Exponential (e.g., base 200ms, factor 2.0), full jitter.
- Caps: Max backoff 30s; per-session ceiling (e.g., 5min window) to avoid unbounded retries if upstream down.
- Reset: On successful message receipt, reset backoff to base.
- Errors: Treat DNS/connect/timeouts/5xx similarly; 4xx may stop depending on semantics.

## Implementation Notes

- Timer control: Use a `tokio::time::Sleep` per connection; avoid shared global timers.
- Cancellation: On shutdown, cancel outstanding sleeps and close streams cleanly.
- Idempotency: Ensure re-subscription does not duplicate handlers; clean up old state before reconnect.
- Observability: Log reconnect attempts with attempt number and backoff; emit metrics for success/failure streaks.

## Tests

- Deterministic timers (`tokio::time::pause` + `advance`).
- Scenarios: transient 5xx, immediate EOF, network flap; verify bounded backoff with jitter distribution.
- No thundering herd: with N clients starting together, ensure jitter produces spread reconnects.

