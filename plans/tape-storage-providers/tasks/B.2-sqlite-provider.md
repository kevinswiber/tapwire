# Task B.2: SQLite Provider Implementation

## Overview
Implement a SQLite-based storage provider for efficient tape storage with advanced querying capabilities.

**Duration**: 4 hours  
**Dependencies**: A.1, A.2, A.3, A.4  
**Status**: ⬜ Not Started

## Objectives

1. Design SQLite schema for tape storage
2. Implement TapeStorageBackend for SQLite
3. Create SqliteProviderFactory
4. Add advanced querying capabilities
5. Implement connection pooling and optimization

## Database Schema Design

### Core Tables

```sql
-- Tapes table
CREATE TABLE tapes (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    transport_type TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    ended_at INTEGER,
    frame_count INTEGER DEFAULT 0,
    total_bytes INTEGER DEFAULT 0,
    metadata_json TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_tapes_session ON tapes(session_id);
CREATE INDEX idx_tapes_started ON tapes(started_at);
CREATE INDEX idx_tapes_created ON tapes(created_at);

-- Frames table (stores message envelopes)
CREATE TABLE frames (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tape_id TEXT NOT NULL,
    sequence_num INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    direction TEXT NOT NULL CHECK(direction IN ('incoming', 'outgoing')),
    message_type TEXT NOT NULL,
    message_json TEXT NOT NULL,
    context_json TEXT,
    size_bytes INTEGER NOT NULL,
    FOREIGN KEY (tape_id) REFERENCES tapes(id) ON DELETE CASCADE,
    UNIQUE(tape_id, sequence_num)
);

CREATE INDEX idx_frames_tape ON frames(tape_id);
CREATE INDEX idx_frames_timestamp ON frames(timestamp);

-- Metadata table for provider-specific data
CREATE TABLE tape_metadata (
    tape_id TEXT PRIMARY KEY,
    compression_type TEXT,
    encryption_status TEXT,
    tags TEXT,  -- JSON array of tags
    custom_data TEXT,  -- JSON object
    FOREIGN KEY (tape_id) REFERENCES tapes(id) ON DELETE CASCADE
);

-- Search index table
CREATE VIRTUAL TABLE tape_search USING fts5(
    tape_id,
    content,
    tokenize='porter'
);
```

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// Database file path
    pub database_path: PathBuf,
    
    /// Connection pool settings
    #[serde(default)]
    pub connection_pool: PoolConfig,
    
    /// Enable WAL mode
    #[serde(default = "default_wal_mode")]
    pub wal_mode: bool,
    
    /// Enable foreign keys
    #[serde(default = "default_foreign_keys")]
    pub foreign_keys: bool,
    
    /// Busy timeout in milliseconds
    #[serde(default = "default_busy_timeout")]
    pub busy_timeout: u32,
    
    /// Enable full-text search
    #[serde(default)]
    pub enable_search: bool,
    
    /// Vacuum schedule
    #[serde(default)]
    pub auto_vacuum: AutoVacuumMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoVacuumMode {
    None,
    Full,
    Incremental,
}
```

## SQLite Provider Implementation

```rust
use sqlx::{SqlitePool, SqliteConnection, Row};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub struct SqliteProvider {
    pool: SqlitePool,
    config: SqliteConfig,
    search_enabled: bool,
}

#[async_trait]
impl TapeStorageBackend for SqliteProvider {
    type Config = SqliteConfig;
    type Metadata = SqliteMetadata;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<()> {
        self.config = config;
        
        // Create database connection pool
        let options = SqliteConnectOptions::new()
            .filename(&self.config.database_path)
            .create_if_missing(true)
            .foreign_keys(self.config.foreign_keys)
            .busy_timeout(Duration::from_millis(self.config.busy_timeout as u64));
        
        if self.config.wal_mode {
            options = options.journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        }
        
        let pool = SqlitePoolOptions::new()
            .max_connections(self.config.connection_pool.max_connections)
            .min_connections(self.config.connection_pool.min_connections)
            .connect_timeout(Duration::from_secs(self.config.connection_pool.connection_timeout))
            .connect_with(options)
            .await?;
        
        self.pool = pool;
        
        // Run migrations
        self.run_migrations().await?;
        
        // Initialize search if enabled
        if self.config.enable_search {
            self.initialize_search().await?;
        }
        
        Ok(())
    }
    
    async fn save_tape(&mut self, tape: &Tape) -> Result<TapeId> {
        let mut tx = self.pool.begin().await?;
        
        // Insert tape record
        sqlx::query(
            r#"
            INSERT INTO tapes (
                id, session_id, transport_type, started_at, ended_at,
                frame_count, total_bytes, metadata_json, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                ended_at = excluded.ended_at,
                frame_count = excluded.frame_count,
                total_bytes = excluded.total_bytes,
                metadata_json = excluded.metadata_json,
                updated_at = excluded.updated_at
            "#
        )
        .bind(&tape.id)
        .bind(&tape.session_id)
        .bind(&tape.transport_type.to_string())
        .bind(tape.started_at.timestamp())
        .bind(tape.ended_at.map(|t| t.timestamp()))
        .bind(tape.frames.len() as i64)
        .bind(tape.total_bytes as i64)
        .bind(serde_json::to_string(&tape.metadata)?)
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .execute(&mut tx)
        .await?;
        
        // Insert frames
        for (seq, frame) in tape.frames.iter().enumerate() {
            self.insert_frame(&mut tx, &tape.id, seq as i64, frame).await?;
        }
        
        // Update search index if enabled
        if self.search_enabled {
            self.update_search_index(&mut tx, &tape.id, tape).await?;
        }
        
        tx.commit().await?;
        
        Ok(tape.id.clone())
    }
    
    async fn load_tape(&self, tape_id: &TapeId) -> Result<Tape> {
        // Load tape metadata
        let tape_row = sqlx::query(
            r#"
            SELECT id, session_id, transport_type, started_at, ended_at,
                   frame_count, total_bytes, metadata_json
            FROM tapes
            WHERE id = ?
            "#
        )
        .bind(tape_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| StorageError::TapeNotFound(tape_id.clone()))?;
        
        // Load frames
        let frames = sqlx::query(
            r#"
            SELECT sequence_num, timestamp, direction, message_type,
                   message_json, context_json
            FROM frames
            WHERE tape_id = ?
            ORDER BY sequence_num
            "#
        )
        .bind(tape_id)
        .fetch_all(&self.pool)
        .await?;
        
        // Reconstruct tape
        let mut tape = Tape {
            id: tape_row.get("id"),
            session_id: tape_row.get("session_id"),
            transport_type: tape_row.get::<String, _>("transport_type").parse()?,
            started_at: DateTime::from_timestamp(tape_row.get("started_at"), 0).unwrap(),
            ended_at: tape_row.get::<Option<i64>, _>("ended_at")
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
            frames: Vec::new(),
            total_bytes: tape_row.get::<i64, _>("total_bytes") as usize,
            metadata: serde_json::from_str(tape_row.get("metadata_json"))?,
        };
        
        // Reconstruct frames
        for frame_row in frames {
            let envelope = self.reconstruct_envelope(frame_row)?;
            tape.frames.push(envelope);
        }
        
        Ok(tape)
    }
    
    async fn delete_tape(&mut self, tape_id: &TapeId) -> Result<()> {
        let result = sqlx::query("DELETE FROM tapes WHERE id = ?")
            .bind(tape_id)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(StorageError::TapeNotFound(tape_id.clone()));
        }
        
        Ok(())
    }
    
    async fn list_tapes(&self) -> Result<Vec<TapeMetadata>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.session_id, t.started_at, t.ended_at,
                   t.frame_count, t.total_bytes, m.tags
            FROM tapes t
            LEFT JOIN tape_metadata m ON t.id = m.tape_id
            ORDER BY t.created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut metadata = Vec::new();
        for row in rows {
            metadata.push(TapeMetadata {
                id: row.get("id"),
                session_id: row.get("session_id"),
                started_at: DateTime::from_timestamp(row.get("started_at"), 0).unwrap(),
                ended_at: row.get::<Option<i64>, _>("ended_at")
                    .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
                frame_count: row.get::<i64, _>("frame_count") as usize,
                size_bytes: row.get::<i64, _>("total_bytes") as usize,
                tags: row.get::<Option<String>, _>("tags")
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
            });
        }
        
        Ok(metadata)
    }
    
    async fn find_by_session(&self, session_id: &SessionId) -> Result<Vec<TapeId>> {
        let rows = sqlx::query("SELECT id FROM tapes WHERE session_id = ?")
            .bind(session_id.to_string())
            .fetch_all(&self.pool)
            .await?;
        
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }
    
    fn storage_type(&self) -> &str {
        "sqlite"
    }
}
```

### Advanced Features

```rust
impl SqliteProvider {
    /// Full-text search across tape contents
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        if !self.search_enabled {
            return Err(StorageError::OperationFailed(
                "Search not enabled".to_string()
            ));
        }
        
        let rows = sqlx::query(
            r#"
            SELECT tape_id, snippet(tape_search, 1, '<b>', '</b>', '...', 32) as snippet
            FROM tape_search
            WHERE tape_search MATCH ?
            ORDER BY rank
            LIMIT 100
            "#
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;
        
        // Convert to search results
        Ok(rows.into_iter().map(|row| SearchResult {
            tape_id: row.get("tape_id"),
            snippet: row.get("snippet"),
        }).collect())
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as tape_count,
                SUM(frame_count) as total_frames,
                SUM(total_bytes) as total_bytes,
                MIN(started_at) as earliest,
                MAX(ended_at) as latest
            FROM tapes
            "#
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(StorageStats {
            tape_count: row.get::<i64, _>("tape_count") as usize,
            total_frames: row.get::<Option<i64>, _>("total_frames")
                .unwrap_or(0) as usize,
            total_bytes: row.get::<Option<i64>, _>("total_bytes")
                .unwrap_or(0) as usize,
            earliest_tape: row.get::<Option<i64>, _>("earliest")
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
            latest_tape: row.get::<Option<i64>, _>("latest")
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap()),
        })
    }
}
```

## Factory Implementation

```rust
pub struct SqliteProviderFactory;

#[async_trait]
impl StorageProviderFactory for SqliteProviderFactory {
    fn name(&self) -> &str {
        "sqlite"
    }
    
    fn display_name(&self) -> &str {
        "SQLite Storage"
    }
    
    fn description(&self) -> &str {
        "High-performance SQLite database storage with full-text search and advanced querying"
    }
    
    async fn create(&self, config: Value) -> Result<Box<dyn TapeStorageBackend>> {
        let sqlite_config: SqliteConfig = serde_json::from_value(config)?;
        
        let mut provider = SqliteProvider::new();
        provider.initialize(sqlite_config).await?;
        
        Ok(Box::new(provider))
    }
    
    fn validate_config(&self, config: &Value) -> Result<()> {
        let _: SqliteConfig = serde_json::from_value(config.clone())?;
        Ok(())
    }
}
```

## Performance Optimizations

1. **Connection pooling**: Reuse connections for better performance
2. **WAL mode**: Enable write-ahead logging for concurrent access
3. **Prepared statements**: Cache and reuse statements
4. **Batch inserts**: Insert multiple frames in one transaction
5. **Indexes**: Strategic indexes for common queries
6. **Vacuum**: Periodic maintenance for optimal performance

## Testing Strategy

- Schema creation and migration
- CRUD operations
- Transaction handling
- Concurrent access
- Full-text search
- Performance benchmarks
- Connection pool behavior

## Success Criteria

- [ ] Schema properly designed
- [ ] All CRUD operations work
- [ ] Search functionality works
- [ ] Connection pooling efficient
- [ ] Transactions are atomic
- [ ] Performance meets targets
- [ ] Migration from filesystem works
- [ ] Tests pass

## Notes

- Consider compression at column level
- Think about partitioning for large datasets
- May need archive/purge strategies
- Consider replication for backup
- Watch for lock contention

## References

- sqlx documentation
- SQLite best practices
- FTS5 full-text search documentation

---

**Next Task**: B.3 - Provider Testing