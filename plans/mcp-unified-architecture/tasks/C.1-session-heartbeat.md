# Task C.1: Session Heartbeat and Liveness Detection

## Objective
Implement proactive session liveness detection to prevent orphaned sessions from consuming resources when connections die without clean disconnection.

## Background
From Gemini's review: The current cleanup strategy is primarily time-based (idle time, max age) or reactive (memory pressure). It doesn't explicitly account for scenarios where the consuming application or the underlying transport disappears without a clean disconnect.

## Key Requirements

### 1. Connection Liveness Check
- Extend the `Connection` trait with liveness detection:
  ```rust
  #[async_trait]
  pub trait Connection: Send + Sync + 'static {
      // Existing methods...
      
      /// Check if the underlying connection is still alive
      /// Returns false if the connection is definitely dead
      /// Returns true if alive or uncertain (to avoid false positives)
      async fn is_alive(&self) -> bool;
  }
  ```

### 2. Implementation for Different Connection Types

#### Hyper Connections
- Use `h2::PingPong` for HTTP/2 connections
- For HTTP/1.1, attempt a TCP-level keepalive check
- Check if the underlying socket is still readable/writable

#### WebSocket Connections  
- Send WebSocket ping frames
- Track pong responses with timeout

#### Stdio Connections
- Check if stdin/stdout file descriptors are still valid
- Detect if the parent process has exited

### 3. SessionManager Heartbeat Task
```rust
impl SessionManager {
    /// Start a background task that periodically checks session liveness
    pub fn start_heartbeat_task(&self) -> JoinHandle<()> {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                manager.check_session_liveness().await;
            }
        })
    }
    
    async fn check_session_liveness(&self) {
        let sessions = self.store.read().await;
        let mut dead_sessions = Vec::new();
        
        for (id, session) in sessions.iter() {
            if let Some(conn) = session.connection() {
                if !conn.is_alive().await {
                    dead_sessions.push(id.clone());
                }
            }
        }
        
        // Remove dead sessions
        for id in dead_sessions {
            tracing::warn!(session_id = %id, "Removing dead session");
            self.remove_session(&id).await;
        }
    }
}
```

### 4. Configuration
Add to `SessionConfig`:
```rust
pub struct SessionConfig {
    // Existing fields...
    
    /// Enable session heartbeat checking
    pub enable_heartbeat: bool,
    
    /// Interval between heartbeat checks
    pub heartbeat_interval: Duration,
    
    /// Timeout for individual liveness checks
    pub liveness_check_timeout: Duration,
}
```

## Implementation Steps

1. **Update Connection trait** (30 min)
   - Add `is_alive()` method
   - Document semantics (false positives are acceptable)

2. **Implement for Hyper** (2 hours)
   - HTTP/2 ping implementation
   - HTTP/1.1 TCP keepalive
   - Handle connection pools properly

3. **Implement for WebSocket** (1 hour)
   - WebSocket ping/pong mechanism
   - Track outstanding pings

4. **Implement for Stdio** (1 hour)
   - File descriptor validation
   - Parent process detection

5. **Add heartbeat task to SessionManager** (1 hour)
   - Configurable interval
   - Batch liveness checks for efficiency
   - Proper error handling and logging

6. **Add metrics** (30 min)
   - `mcp_session_heartbeat_checks_total`
   - `mcp_session_dead_detected_total`
   - `mcp_session_liveness_check_duration_seconds`

## Testing Strategy

1. **Unit Tests**
   - Mock connections that report dead/alive
   - Verify dead sessions are removed
   - Test heartbeat task scheduling

2. **Integration Tests**
   - Kill upstream connection abruptly
   - Verify session cleanup within heartbeat interval
   - Test with different connection types

3. **Chaos Testing**
   - Network partition simulation
   - Half-open connection scenarios
   - Process crash simulation

## Success Criteria

- [ ] Dead connections detected within 2x heartbeat interval
- [ ] No false positives causing live session termination
- [ ] Heartbeat overhead < 1% CPU usage
- [ ] All connection types support liveness checking
- [ ] Metrics show heartbeat effectiveness

## Risk Mitigation

1. **False Positives**: Conservative implementation - when uncertain, assume alive
2. **Performance Impact**: Batch checks, use async operations
3. **Connection Pool Complexity**: Coordinate with pool's own health checks

## Dependencies
- Task C.0 (SessionManager implementation)
- Connection trait must be established

## Estimated Duration
6 hours

## Notes
- Consider using `tokio::net::TcpStream::peek()` for TCP liveness
- HTTP/2 ping frames are built into the protocol
- WebSocket ping/pong is also protocol-native
- For stdio, we might need platform-specific code (epoll on Linux, kqueue on macOS)