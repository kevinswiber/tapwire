# MCP-Aware Recorder Specification

## Overview

This specification details the upgrade of Shadowcat's recording system from binary transport recording to semantic MCP message recording with full session context.

## Recording Architecture

### Current: Binary Transport Recording
```
[Transport Bytes] → [Timestamp] → [Binary Storage]
```

### Target: Semantic MCP Recording
```
[MCP Messages] → [Parse & Correlate] → [Enrich] → [Structured Storage]
                           ↓                ↓              ↓
                    [Session Context]  [Metadata]  [Search Index]
```

## Core Components

### 1. MCP Tape Format

**Purpose**: Define the structure for recorded MCP sessions

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTape {
    // Identity
    pub id: TapeId,
    pub name: String,
    pub description: Option<String>,
    
    // Session info
    pub session_id: SessionId,
    pub mcp_session_id: Option<String>,
    pub protocol_version: String,
    
    // Timing
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    
    // Content
    pub entries: Vec<TapeEntry>,
    
    // Metadata
    pub metadata: TapeMetadata,
    
    // Recording configuration
    pub config: RecordingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeEntry {
    // Unique entry ID
    pub id: EntryId,
    
    // Timing
    pub timestamp: DateTime<Utc>,
    pub relative_time: Duration,  // Since tape start
    
    // Message
    pub message: RecordedMessage,
    
    // Context
    pub direction: MessageDirection,
    pub transport: TransportType,
    pub correlation_id: Option<CorrelationId>,
    
    // Processing info
    pub intercepted: bool,
    pub original_message: Option<McpMessage>,  // If modified
    pub applied_rules: Vec<RuleId>,
    
    // Performance
    pub processing_time: Duration,
    pub response_time: Option<Duration>,  // For correlated req/resp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedMessage {
    pub raw: McpMessage,
    pub parsed: ParsedMessage,
    pub size_bytes: usize,
    pub compression: Option<CompressionType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMessage {
    pub message_type: MessageType,
    pub method: Option<String>,
    pub id: Option<JsonRpcId>,
    pub params_summary: Option<ParamsSummary>,
    pub result_summary: Option<ResultSummary>,
    pub error_info: Option<ErrorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeMetadata {
    // Statistics
    pub total_messages: usize,
    pub total_requests: usize,
    pub total_responses: usize,
    pub total_notifications: usize,
    pub total_errors: usize,
    
    // Method analysis
    pub methods: HashMap<String, MethodStats>,
    pub method_pairs: HashMap<(String, String), usize>,  // Sequential patterns
    
    // Performance
    pub avg_response_time: Duration,
    pub p50_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    
    // Session info
    pub client_info: Option<ClientInfo>,
    pub server_info: Option<ServerInfo>,
    
    // Data volume
    pub total_bytes_sent: usize,
    pub total_bytes_received: usize,
    
    // Errors
    pub error_rate: f64,
    pub error_codes: HashMap<i32, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodStats {
    pub count: usize,
    pub success_count: usize,
    pub error_count: usize,
    pub avg_response_time: Duration,
    pub avg_params_size: usize,
    pub avg_result_size: usize,
}
```

### 2. Recording Engine

**Purpose**: Capture and process MCP messages for recording

```rust
pub struct McpRecorder {
    active_tapes: HashMap<SessionId, ActiveTape>,
    storage: Box<dyn TapeStorage>,
    config: RecorderConfig,
    correlator: MessageCorrelator,
    enricher: MessageEnricher,
}

pub struct ActiveTape {
    tape: McpTape,
    buffer: Vec<TapeEntry>,
    stats_collector: StatsCollector,
    last_flush: Instant,
}

pub struct RecorderConfig {
    // Filtering
    pub record_requests: bool,
    pub record_responses: bool,
    pub record_notifications: bool,
    pub filter_methods: Option<HashSet<String>>,
    pub exclude_methods: Option<HashSet<String>>,
    
    // Sampling
    pub sampling_rate: f64,  // 0.0 to 1.0
    pub sampling_strategy: SamplingStrategy,
    
    // Storage
    pub max_tape_size: usize,
    pub max_tape_duration: Duration,
    pub compression: CompressionType,
    pub flush_interval: Duration,
    pub buffer_size: usize,
    
    // Privacy
    pub redact_sensitive: bool,
    pub sensitive_fields: Vec<String>,
    pub hash_ids: bool,
}

pub enum SamplingStrategy {
    Random,
    Systematic { interval: usize },
    Adaptive { target_rate: f64 },
    Priority { rules: Vec<SamplingRule> },
}

impl McpRecorder {
    pub async fn record_message(
        &mut self,
        message: McpMessage,
        context: MessageContext,
    ) -> Result<EntryId> {
        // Check if we should record
        if !self.should_record(&message, &context) {
            return Ok(EntryId::skipped());
        }
        
        // Get or create tape for session
        let tape = self.active_tapes
            .entry(context.session_id.clone())
            .or_insert_with(|| self.create_tape(context.session_id.clone()));
        
        // Create entry
        let entry = self.create_entry(message, context).await?;
        
        // Update statistics
        tape.stats_collector.update(&entry);
        
        // Add to buffer
        tape.buffer.push(entry.clone());
        
        // Flush if needed
        if self.should_flush(tape) {
            self.flush_tape(context.session_id).await?;
        }
        
        Ok(entry.id)
    }
    
    async fn create_entry(
        &mut self,
        message: McpMessage,
        context: MessageContext,
    ) -> Result<TapeEntry> {
        // Parse message
        let parsed = parse_mcp_message(&message)?;
        
        // Correlate if response
        let correlation_id = if let Some(id) = extract_message_id(&message) {
            self.correlator.get_correlation_id(id).await
        } else {
            None
        };
        
        // Calculate response time if correlated
        let response_time = if let Some(cid) = &correlation_id {
            self.correlator.get_response_time(cid).await
        } else {
            None
        };
        
        // Enrich with additional context
        let enriched = self.enricher.enrich(message.clone(), &context).await?;
        
        // Apply privacy settings
        let final_message = if self.config.redact_sensitive {
            self.redact_sensitive_data(enriched)?
        } else {
            enriched
        };
        
        Ok(TapeEntry {
            id: EntryId::new(),
            timestamp: Utc::now(),
            relative_time: context.relative_time,
            message: RecordedMessage {
                raw: final_message,
                parsed,
                size_bytes: estimate_message_size(&message),
                compression: None,  // Compressed on storage
            },
            direction: context.direction,
            transport: context.transport,
            correlation_id,
            intercepted: context.intercepted,
            original_message: context.original_message,
            applied_rules: context.applied_rules,
            processing_time: context.processing_time,
            response_time,
        })
    }
}
```

### 3. Storage Backend

**Purpose**: Efficient storage and retrieval of MCP tapes

```rust
pub trait TapeStorage: Send + Sync {
    async fn create_tape(&self, tape: McpTape) -> Result<TapeId>;
    async fn append_entries(&self, tape_id: TapeId, entries: Vec<TapeEntry>) -> Result<()>;
    async fn finalize_tape(&self, tape_id: TapeId, metadata: TapeMetadata) -> Result<()>;
    async fn get_tape(&self, tape_id: TapeId) -> Result<Option<McpTape>>;
    async fn list_tapes(&self, filter: TapeFilter) -> Result<Vec<TapeSummary>>;
    async fn delete_tape(&self, tape_id: TapeId) -> Result<()>;
    async fn search_entries(&self, query: SearchQuery) -> Result<Vec<TapeEntry>>;
}

pub struct SqliteTapeStorage {
    pool: SqlitePool,
    compression: CompressionType,
}

impl SqliteTapeStorage {
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tapes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                session_id TEXT NOT NULL,
                mcp_session_id TEXT,
                protocol_version TEXT NOT NULL,
                started_at TIMESTAMP NOT NULL,
                ended_at TIMESTAMP,
                metadata BLOB,
                config BLOB,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_session_id (session_id),
                INDEX idx_started_at (started_at)
            );
            
            CREATE TABLE IF NOT EXISTS tape_entries (
                id TEXT PRIMARY KEY,
                tape_id TEXT NOT NULL,
                timestamp TIMESTAMP NOT NULL,
                relative_time_ms INTEGER NOT NULL,
                message_type TEXT NOT NULL,
                method TEXT,
                jsonrpc_id TEXT,
                direction TEXT NOT NULL,
                transport TEXT NOT NULL,
                correlation_id TEXT,
                message_data BLOB NOT NULL,
                metadata BLOB,
                FOREIGN KEY (tape_id) REFERENCES tapes(id) ON DELETE CASCADE,
                INDEX idx_tape_id (tape_id),
                INDEX idx_timestamp (timestamp),
                INDEX idx_method (method),
                INDEX idx_correlation (correlation_id)
            );
            
            CREATE VIRTUAL TABLE IF NOT EXISTS tape_search USING fts5(
                entry_id,
                tape_id,
                method,
                params_text,
                result_text,
                error_text
            );
            "#
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn store_entry(&self, tape_id: TapeId, entry: TapeEntry) -> Result<()> {
        // Compress message data
        let compressed = self.compress_message(&entry.message)?;
        
        // Store main entry
        sqlx::query(
            r#"
            INSERT INTO tape_entries (
                id, tape_id, timestamp, relative_time_ms,
                message_type, method, jsonrpc_id,
                direction, transport, correlation_id,
                message_data, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&entry.id)
        .bind(&tape_id)
        .bind(&entry.timestamp)
        .bind(entry.relative_time.as_millis() as i64)
        .bind(&entry.message.parsed.message_type.to_string())
        .bind(&entry.message.parsed.method)
        .bind(&entry.message.parsed.id.map(|id| id.to_string()))
        .bind(&entry.direction.to_string())
        .bind(&entry.transport.to_string())
        .bind(&entry.correlation_id)
        .bind(&compressed)
        .bind(&serialize_metadata(&entry)?)
        .execute(&self.pool)
        .await?;
        
        // Update search index
        self.update_search_index(tape_id, entry).await?;
        
        Ok(())
    }
}
```

### 4. Search and Query

**Purpose**: Enable powerful searching within recorded sessions

```rust
pub struct TapeSearchEngine {
    storage: Arc<dyn TapeStorage>,
    indexer: MessageIndexer,
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    // Time range
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    
    // Message filters
    pub methods: Option<Vec<String>>,
    pub message_types: Option<Vec<MessageType>>,
    pub session_ids: Option<Vec<SessionId>>,
    
    // Content search
    pub text_search: Option<String>,
    pub param_filters: Vec<ParamFilter>,
    pub result_filters: Vec<ResultFilter>,
    
    // Error filters
    pub error_codes: Option<Vec<i32>>,
    pub has_errors: Option<bool>,
    
    // Performance filters
    pub min_response_time: Option<Duration>,
    pub max_response_time: Option<Duration>,
    
    // Pagination
    pub offset: usize,
    pub limit: usize,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone)]
pub enum ParamFilter {
    FieldExists(String),
    FieldEquals(String, Value),
    FieldContains(String, String),
    FieldGreaterThan(String, f64),
    FieldLessThan(String, f64),
}

impl TapeSearchEngine {
    pub async fn search(&self, query: SearchQuery) -> Result<SearchResults> {
        // Build SQL query
        let sql = self.build_search_query(&query)?;
        
        // Execute search
        let entries = self.storage.search_entries(sql).await?;
        
        // Post-process results
        let processed = self.post_process_results(entries, &query)?;
        
        // Calculate facets
        let facets = self.calculate_facets(&processed)?;
        
        Ok(SearchResults {
            entries: processed,
            total_count: facets.total_count,
            facets,
            query,
        })
    }
    
    pub async fn analyze_tape(&self, tape_id: TapeId) -> Result<TapeAnalysis> {
        let tape = self.storage.get_tape(tape_id).await?
            .ok_or_else(|| anyhow!("Tape not found"))?;
        
        Ok(TapeAnalysis {
            // Message flow
            message_flow: self.analyze_message_flow(&tape),
            
            // Method patterns
            method_sequences: self.find_method_patterns(&tape),
            common_errors: self.analyze_errors(&tape),
            
            // Performance analysis
            performance_profile: self.analyze_performance(&tape),
            bottlenecks: self.identify_bottlenecks(&tape),
            
            // Anomalies
            anomalies: self.detect_anomalies(&tape),
        })
    }
}
```

### 5. Privacy and Redaction

**Purpose**: Protect sensitive data in recordings

```rust
pub struct PrivacyManager {
    rules: Vec<RedactionRule>,
    hasher: Sha256,
}

#[derive(Debug, Clone)]
pub struct RedactionRule {
    pub name: String,
    pub pattern: RedactionPattern,
    pub action: RedactionAction,
}

#[derive(Debug, Clone)]
pub enum RedactionPattern {
    FieldPath(String),  // JSONPath expression
    Regex(Regex),
    MethodParam { method: String, param: String },
    Custom(Box<dyn Fn(&Value) -> bool>),
}

#[derive(Debug, Clone)]
pub enum RedactionAction {
    Remove,
    Replace(Value),
    Hash,
    Mask { visible_chars: usize },
    Encrypt { key_id: String },
}

impl PrivacyManager {
    pub fn redact_message(&self, mut message: McpMessage) -> Result<McpMessage> {
        match &mut message {
            McpMessage::Single(msg) => self.redact_single(msg)?,
            McpMessage::Batch(messages) => {
                for msg in messages {
                    self.redact_single(msg)?;
                }
            }
        }
        Ok(message)
    }
    
    fn redact_single(&self, message: &mut JsonRpcMessage) -> Result<()> {
        if let JsonRpcMessage::V2(msg) = message {
            match msg {
                JsonRpcV2Message::Request { params, .. } |
                JsonRpcV2Message::Notification { params, .. } => {
                    if let Some(params) = params {
                        self.apply_redaction_rules(params)?;
                    }
                }
                JsonRpcV2Message::Response { result, .. } => {
                    if let Some(result) = result {
                        self.apply_redaction_rules(result)?;
                    }
                }
            }
        }
        Ok(())
    }
}
```

## Recording Strategies

### 1. Full Recording
- Record every message
- No sampling or filtering
- Maximum detail for debugging

### 2. Sampled Recording
- Record percentage of sessions
- Or record specific time windows
- Balance between coverage and storage

### 3. Error-Focused Recording
- Always record errors
- Record N messages before/after errors
- Minimal storage, maximum debugging value

### 4. Performance Recording
- Record slow requests (> threshold)
- Include timing metadata
- Focus on optimization opportunities

## Configuration

```yaml
recorder:
  enabled: true
  
  # What to record
  capture:
    requests: true
    responses: true
    notifications: true
    errors: true  # Always record errors
  
  # Filtering
  filters:
    include_methods:
      - "tools/*"
      - "resources/*"
    exclude_methods:
      - "ping"
      - "*/list"
    min_response_time: 100ms  # Only record slow requests
  
  # Sampling
  sampling:
    strategy: adaptive
    target_rate: 0.1  # Record 10% of traffic
    always_sample:
      - errors
      - slow_requests
  
  # Storage
  storage:
    backend: sqlite
    path: ./tapes
    max_tape_size: 100MB
    max_tape_duration: 1h
    compression: zstd
    retention: 30d
  
  # Privacy
  privacy:
    redact_sensitive: true
    sensitive_fields:
      - password
      - api_key
      - token
      - secret
    hash_ids: true
    encryption:
      enabled: false
      key_id: "default"
  
  # Performance
  performance:
    buffer_size: 1000
    flush_interval: 5s
    async_storage: true
```

## Export Formats

### 1. Native Binary Format
- Most efficient storage
- Fast replay
- Requires Shadowcat to read

### 2. JSON Lines
- Human readable
- One message per line
- Easy to process with standard tools

### 3. HAR (HTTP Archive)
- Standard format for HTTP traffic
- Compatible with browser dev tools
- Good for HTTP/SSE transports

### 4. OpenTelemetry
- Standard observability format
- Integration with APM tools
- Distributed tracing support

## Future Enhancements

1. **Cloud Storage**
   - S3/GCS/Azure Blob support
   - Distributed recording across instances

2. **Real-time Streaming**
   - Stream recordings to analysis tools
   - Live debugging capabilities

3. **AI-Powered Analysis**
   - Automatic pattern recognition
   - Anomaly detection
   - Performance optimization suggestions

4. **Compliance Features**
   - Audit logging
   - Retention policies
   - GDPR compliance tools