# SSE Reconnection: Existing Module vs. Plan

Date: 2025-08-19

## Observation

- A comprehensive SSE reconnection module exists at `shadowcat/src/transport/sse/reconnect.rs` implementing a state machine with backoff, deduplication, and health monitoring.
- The Phase H task (H.4) targets `src/proxy/reverse/upstream/http/streaming/intercepted.rs` for implementing reconnection logic.

## Risk

- Duplicating reconnection logic across two places (transport module and reverse upstream stream) can cause divergence and maintenance burden.

## Recommendation

- Prefer integrating the existing `transport::sse::reconnect` components into the reverse upstream streaming path rather than re-implementing.
- If the reverse layer has additional concerns (session, auth headers, Last-Event-Id), adapt the reconnection module to accept hooks/callbacks for request construction and post-reconnect notifications.
- Add a small adapter layer in `reverse/upstream/http/streaming` that:
  - Supplies a “connect” function returning an SSE stream given headers/URL.
  - Supplies a “last_event_id accessor/updater”.
  - Subscribes to reconnection events to emit metrics and client notifications.

## Acceptance

- Single, unified reconnection implementation used across proxy components.
- Tests for reverse path reuse the transport reconnection test harness with minimal duplication.

