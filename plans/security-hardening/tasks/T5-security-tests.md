# T5: Security Tests

Objective: Implement the test plan for header scrubbing and allowlists.

## Implementation
- Add integration tests for HTTP/SSE reverse paths
- Add unit tests for header builders

## Target Areas
- `shadowcat/tests/security_http.rs`
- `shadowcat/tests/security_sse.rs`
- Existing builders in reverse proxy

## Done When
- Tests pass locally and in CI; coverage captures sensitive header non-propagation
