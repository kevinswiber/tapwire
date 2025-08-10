### Hot-path allocation and logging audit (Delta: shadowcat-delta@b793fd1)

- Strong indicators of per-frame allocations and verbose logging inside tight loops in transports and proxy flows.

Findings

- Stdio transport I/O loops clone and log full lines per message
```71:101:shadowcat-delta/src/transport/stdio.rs
            while let Some(msg) = stdin_rx.recv().await {
                debug!("Writing to stdin: {}", msg);
                // … size checks …
                if let Err(e) = stdin.write_all(msg.as_bytes()).await { … }
                if let Err(e) = stdin.write_all(b"\n").await { … }
                if let Err(e) = stdin.flush().await { … }
            }
```
```108:144:shadowcat-delta/src/transport/stdio.rs
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => { … break; }
                    Ok(bytes_read) => {
                        // trim newline
                        debug!("Read from stdout: {}", line);
                        if stdout_tx.send(line.clone()).await.is_err() { … break; }
                    }
                    Err(e) => { … break; }
                }
            }
```
- Stdio message parsing allocates strings for ids/methods
```179:211:shadowcat-delta/src/transport/stdio.rs
            let method_str = method.as_str().ok_or_else(|| … )?;
            let params = json_value.get("params").cloned().unwrap_or(Value::Null);
            if let Some(id) = json_value.get("id") {
                let id_str = match id {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    _ => return Err(…)
                };
                let msg = ProtocolMessage::Request { id: id_str, method: method_str.to_string(), params };
                …
```
- Send path logs whole message and allocates serialized buffers
```312:336:shadowcat-delta/src/transport/stdio.rs
        let serialized = self.serialize_message(&envelope.message)?;
        if serialized.len() > self.config.max_message_size { … }
        debug!("Sending message: {:?}", envelope.message);
        timeout(send_timeout, stdin_tx.send(serialized)).await? …
```
- Forward proxy duplicates envelopes and logs each hop
```521:531:shadowcat-delta/src/proxy/forward.rs
        if let Some(session_manager) = &processors.session_manager {
            if let Err(e) = session_manager.record_frame(envelope.clone()).await { … }
        }
        if let Some(tape_recorder) = &processors.tape_recorder {
            if let Err(e) = tape_recorder.record_frame(envelope.clone()).await { … }
        }
```
```634:646:shadowcat-delta/src/proxy/forward.rs
        while let Some(envelope) = message_rx.recv().await {
            debug!("Sending message to {}: {:?}", destination, envelope);
            let result = { let mut transport_guard = transport.write().await; transport_guard.send(envelope).await };
            if let Err(e) = result { error!(…); break; }
        }
```
- Interceptor context enrich fetches session and serializes tags per message
```545:559:shadowcat-delta/src/proxy/forward.rs
            if let Some(session_manager) = &processors.session_manager {
                if let Ok(session) = session_manager.get_session(&context.session_id).await {
                    intercept_context.metadata.insert("frame_count".to_string(), session.frame_count.to_string());
                    intercept_context.metadata.insert(
                        "session_duration_ms".to_string(), session.duration_ms().to_string(),
                    );
                    if !session.tags.is_empty() {
                        intercept_context.metadata.insert(
                            "session_tags".to_string(), serde_json::to_string(&session.tags).unwrap_or_default(),
                        );
                    }
                }
            }
```
- SSE transport double-converts and rebuilds headers every send
```298:307:shadowcat-delta/src/transport/sse_transport.rs
        let json_str = serde_json::to_string(&json_msg)
            .map_err(|e| TransportError::SendFailed(format!("Failed to serialize message: {e}")))?;
        let parsed_info = self.parser.parse(&json_str)
            .map_err(|e| TransportError::SendFailed(format!("Failed to parse message: {e:?}")))?;
```
```330:341:shadowcat-delta/src/transport/sse_transport.rs
        let mut headers = HeaderMap::new();
        headers.insert("MCP-Session-Id", self.session_id.to_string().parse().unwrap());
        headers.insert("MCP-Protocol-Version", self.config.protocol_version.as_str().parse().unwrap());
        headers.insert("X-Event-Id", event_id.parse().unwrap());
```
- SSE reconnect hot loop clones events and allocates futures per frame
```992:1011:shadowcat-delta/src/transport/sse/reconnect.rs
                            let tracker = Arc::clone(&this.event_tracker);
                            let id = id.clone();
                            this.async_op = AsyncOperation::CheckingDuplicate {
                                event: event.clone(),
                                future: Box::pin(async move { tracker.is_duplicate(&id).await }),
                            };
                            return self.poll_next(cx);
```
```1003:1009:shadowcat-delta/src/transport/sse/reconnect.rs
                            this.async_op = AsyncOperation::RecordingEvent {
                                event: event.clone(),
                                future: Box::pin(async move { tracker.record_event(&event_clone).await; }),
                            };
```

Recommendations

- Reduce per-message logging in hot loops
  - Guard with `tracing::enabled!(Level::DEBUG)` or lower verbosity; avoid logging full envelopes.
- Avoid redundant clones
  - Pass borrowed references where possible; refactor recording to accept `&MessageEnvelope` and clone once in a single sink.
- Batch and backpressure
  - Replace per-frame `mpsc::Sender` sends with bounded batching where feasible, or reuse buffers from a pool.
- Prebuild immutable send-time artifacts
  - Cache static headers for SSE; avoid `to_string().parse().unwrap()` per send; keep `HeaderMap` template.
- Eliminate double conversions
  - Skip serialize-then-parse in SSE send; feed `ProtocolMessage` directly to parser or trust prior validation.
- Consider sampling metrics/logs
  - Sample at 1:N in hot paths to cut overhead.
