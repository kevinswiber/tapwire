# Task 007: Enhanced Tape Format

**File:** `src/recorder/format.rs`  
**Estimated Effort:** 2 days  
**Priority:** High  
**Dependencies:** TapeRecorder, TapePlayer

---

## Overview

Enhance the tape format with versioning, metadata, compression, and validation capabilities to create a robust, future-proof recording system that can evolve with Shadowcat's needs.

---

## Requirements

### Core Enhancements
1. **Format Versioning**: Support multiple tape format versions with migration
2. **Rich Metadata**: Include environment, session context, and statistics
3. **Integrity Verification**: Checksums and validation for data integrity
4. **Compression Support**: Reduce storage requirements for large tapes
5. **Migration Utilities**: Seamless upgrade from v0 to v1 format

### Extended Features
1. **Index Generation**: Fast access to frames and metadata
2. **Partial Loading**: Stream large tapes without full memory load
3. **Format Validation**: Comprehensive schema checking
4. **Repair Utilities**: Fix corrupted or incomplete tapes
5. **Statistics Collection**: Detailed analytics and insights

---

## Technical Specification

### Version 1 Tape Format
```json
{
  "format": {
    "version": "1.0",
    "schema_version": 1,
    "created_by": "shadowcat-0.1.0",
    "created_at": "2025-08-04T10:30:00.123Z"
  },
  "integrity": {
    "checksum": "sha256:abc123...",
    "compression": "gzip",
    "original_size": 1048576,
    "compressed_size": 262144
  },
  "environment": {
    "platform": {
      "os": "darwin",
      "arch": "arm64", 
      "version": "14.5.0"
    },
    "runtime": {
      "shadowcat_version": "0.1.0",
      "rust_version": "1.75.0",
      "mcp_version": "2025-11-05"
    },
    "configuration": {
      "buffer_size": 8192,
      "timeout_ms": 30000,
      "max_message_size": 1048576
    }
  },
  "session": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "test-session",
    "description": "Recorded MCP session for debugging",
    "tags": ["debug", "mcp", "stdio"],
    "transport_type": "stdio",
    "client_info": {
      "command": "echo",
      "args": ["hello"],
      "working_dir": "/path/to/dir"
    },
    "server_info": {
      "endpoint": "stdio://echo",
      "version": "1.0.0"
    }
  },
  "timeline": {
    "started_at": 1691145000123,
    "ended_at": 1691145005456,
    "duration_ms": 5333,
    "frame_count": 42,
    "total_bytes": 8192
  },
  "statistics": {
    "requests": 21,
    "responses": 20,
    "notifications": 1,
    "errors": 0,
    "avg_request_size": 256,
    "avg_response_size": 128,
    "peak_throughput": 1024
  },
  "index": {
    "frames_by_type": {
      "request": [0, 2, 4, 6, 8],
      "response": [1, 3, 5, 7, 9],
      "notification": [10]
    },
    "timeline_markers": [
      {"timestamp": 1691145000123, "frame": 0, "type": "session_start"},
      {"timestamp": 1691145002500, "frame": 20, "type": "midpoint"},
      {"timestamp": 1691145005456, "frame": 41, "type": "session_end"}
    ]
  },
  "frames": [
    {
      "id": "frame-uuid-1",
      "sequence": 0,
      "timestamp": 1691145000123,
      "direction": "ClientToServer",
      "message": {
        "jsonrpc": "2.0",
        "id": "1",
        "method": "initialize",
        "params": {}
      },
      "metadata": {
        "size": 128,
        "processing_time_us": 250
      }
    }
  ]
}
```

### Migration System
```rust
// src/recorder/format.rs
use serde::{Deserialize, Serialize};
use crate::error::{FormatError, FormatResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeFormatV1 {
    pub format: FormatMetadata,
    pub integrity: IntegrityInfo,
    pub environment: EnvironmentInfo,
    pub session: SessionInfo,
    pub timeline: TimelineInfo,
    pub statistics: StatisticsInfo,
    pub index: IndexInfo,
    pub frames: Vec<FrameV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatMetadata {
    pub version: String,
    pub schema_version: u32,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub trait TapeFormat {
    fn version(&self) -> u32;
    fn validate(&self) -> FormatResult<()>;
    fn checksum(&self) -> FormatResult<String>;
    fn compress(&self) -> FormatResult<Vec<u8>>;
    fn decompress(data: &[u8]) -> FormatResult<Self> where Self: Sized;
}

pub struct TapeMigrator;

impl TapeMigrator {
    pub fn migrate_to_latest(tape_data: &[u8]) -> FormatResult<TapeFormatV1> {
        // Detect format version
        let version = Self::detect_version(tape_data)?;
        
        match version {
            0 => Self::migrate_v0_to_v1(tape_data),
            1 => Self::parse_v1(tape_data),
            v => Err(FormatError::UnsupportedVersion(v)),
        }
    }
    
    fn detect_version(data: &[u8]) -> FormatResult<u32> {
        // Try to parse as V1 first (has format.schema_version field)
        if let Ok(v1_check) = serde_json::from_slice::<serde_json::Value>(data) {
            if let Some(version) = v1_check.get("format")
                .and_then(|f| f.get("schema_version"))
                .and_then(|v| v.as_u64()) {
                return Ok(version as u32);
            }
        }
        
        // Fall back to V0 detection (has metadata.id field at root)
        if let Ok(v0_check) = serde_json::from_slice::<serde_json::Value>(data) {
            if v0_check.get("metadata").is_some() && v0_check.get("frames").is_some() {
                return Ok(0);
            }
        }
        
        Err(FormatError::UnknownFormat)
    }
}
```

---

## Implementation Plan

### Day 1: Format Definition & Validation

#### Morning: Enhanced Format Structure
```rust
// 1. Define V1 format structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub platform: PlatformInfo,
    pub runtime: RuntimeInfo,
    pub configuration: ConfigurationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,           // "darwin", "linux", "windows"
    pub arch: String,         // "arm64", "x86_64"
    pub version: String,      // OS version
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeInfo {
    pub shadowcat_version: String,
    pub rust_version: String,
    pub mcp_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityInfo {
    pub checksum: String,
    pub compression: CompressionType,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

// 2. Implement validation system
impl TapeFormatV1 {
    pub fn validate(&self) -> FormatResult<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Validate format metadata
        if self.format.schema_version != 1 {
            report.add_error("Invalid schema version");
        }
        
        // Validate frame consistency
        if self.frames.len() != self.timeline.frame_count {
            report.add_error("Frame count mismatch");
        }
        
        // Validate timestamps
        for (i, frame) in self.frames.iter().enumerate() {
            if i > 0 && frame.timestamp < self.frames[i-1].timestamp {
                report.add_warning(&format!("Frame {} has timestamp before previous frame", i));
            }
        }
        
        // Validate checksums
        let calculated_checksum = self.calculate_checksum()?;
        if calculated_checksum != self.integrity.checksum {
            report.add_error("Checksum mismatch - tape may be corrupted");
        }
        
        // Validate statistics
        self.validate_statistics(&mut report)?;
        
        if report.has_errors() {
            Err(FormatError::ValidationFailed(report))
        } else {
            Ok(report)
        }
    }
    
    fn validate_statistics(&self, report: &mut ValidationReport) -> FormatResult<()> {
        let actual_requests = self.frames.iter()
            .filter(|f| matches!(f.message, TransportMessage::Request { .. }))
            .count();
        
        if actual_requests != self.statistics.requests {
            report.add_warning(&format!(
                "Request count mismatch: expected {}, found {}", 
                self.statistics.requests, 
                actual_requests
            ));
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }
    
    pub fn add_warning(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
    }
}
```

#### Afternoon: Checksum & Compression
```rust
// 3. Implement integrity checking
use sha2::{Sha256, Digest};
use flate2::{Compression, write::GzEncoder, read::GzDecoder};

impl TapeFormatV1 {
    pub fn calculate_checksum(&self) -> FormatResult<String> {
        // Calculate checksum of frames + metadata (excluding checksum field)
        let mut hasher = Sha256::new();
        
        // Hash frames
        for frame in &self.frames {
            let frame_bytes = serde_json::to_vec(frame)?;
            hasher.update(&frame_bytes);
        }
        
        // Hash critical metadata
        hasher.update(self.session.id.as_bytes());
        hasher.update(&self.timeline.started_at.to_le_bytes());
        hasher.update(&self.timeline.frame_count.to_le_bytes());
        
        let result = hasher.finalize();
        Ok(format!("sha256:{:x}", result))
    }
    
    pub fn compress(&self) -> FormatResult<Vec<u8>> {
        use std::io::Write;
        
        let json_data = serde_json::to_vec(self)?;
        let original_size = json_data.len();
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&json_data)?;
        let compressed = encoder.finish()?;
        
        // Update compression info
        let mut compressed_tape = self.clone();
        compressed_tape.integrity.original_size = original_size;
        compressed_tape.integrity.compressed_size = Some(compressed.len());
        compressed_tape.integrity.compression = CompressionType::Gzip;
        
        Ok(compressed)
    }
    
    pub fn decompress(compressed_data: &[u8]) -> FormatResult<Self> {
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(compressed_data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        let tape: TapeFormatV1 = serde_json::from_slice(&decompressed)?;
        Ok(tape)
    }
}
```

### Day 2: Migration & Advanced Features

#### Morning: Migration System
```rust
// 4. Implement V0 to V1 migration
impl TapeMigrator {
    fn migrate_v0_to_v1(v0_data: &[u8]) -> FormatResult<TapeFormatV1> {
        // Parse V0 format (current Tape struct)
        let v0_tape: crate::recorder::tape::Tape = serde_json::from_slice(v0_data)?;
        
        // Create V1 format with enhanced metadata
        let format = FormatMetadata {
            version: "1.0".to_string(),
            schema_version: 1,
            created_by: format!("shadowcat-{}", env!("CARGO_PKG_VERSION")),
            created_at: chrono::Utc::now(),
        };
        
        let environment = Self::collect_environment_info();
        let session = Self::migrate_session_info(&v0_tape.metadata);
        let timeline = Self::migrate_timeline_info(&v0_tape);
        let statistics = Self::calculate_statistics(&v0_tape);
        let index = Self::build_index(&v0_tape);
        let frames = Self::migrate_frames(&v0_tape.frames);
        
        let mut v1_tape = TapeFormatV1 {
            format,
            integrity: IntegrityInfo {
                checksum: String::new(), // Will be calculated
                compression: CompressionType::None,
                original_size: v0_data.len(),
                compressed_size: None,
            },
            environment,
            session,
            timeline,
            statistics,
            index,
            frames,
        };
        
        // Calculate final checksum
        v1_tape.integrity.checksum = v1_tape.calculate_checksum()?;
        
        Ok(v1_tape)
    }
    
    fn collect_environment_info() -> EnvironmentInfo {
        EnvironmentInfo {
            platform: PlatformInfo {
                os: std::env::consts::OS.to_string(),
                arch: std::env::consts::ARCH.to_string(), 
                version: Self::get_os_version(),
            },
            runtime: RuntimeInfo {
                shadowcat_version: env!("CARGO_PKG_VERSION").to_string(),
                rust_version: Self::get_rust_version(),
                mcp_version: crate::transport::MCP_PROTOCOL_VERSION.to_string(),
            },
            configuration: ConfigurationInfo {
                buffer_size: Default::default(),
                timeout_ms: Default::default(),
                max_message_size: Default::default(),
            },
        }
    }
    
    fn calculate_statistics(v0_tape: &crate::recorder::tape::Tape) -> StatisticsInfo {
        let mut requests = 0;
        let mut responses = 0;
        let mut notifications = 0;
        let mut total_request_size = 0;
        let mut total_response_size = 0;
        
        for frame in &v0_tape.frames {
            let frame_size = serde_json::to_string(&frame.message)
                .map(|s| s.len())
                .unwrap_or(0);
                
            match &frame.message {
                TransportMessage::Request { .. } => {
                    requests += 1;
                    total_request_size += frame_size;
                }
                TransportMessage::Response { .. } => {
                    responses += 1;
                    total_response_size += frame_size;
                }
                TransportMessage::Notification { .. } => {
                    notifications += 1;
                }
            }
        }
        
        StatisticsInfo {
            requests,
            responses,
            notifications,
            errors: 0, // TODO: Detect error responses
            avg_request_size: if requests > 0 { total_request_size / requests } else { 0 },
            avg_response_size: if responses > 0 { total_response_size / responses } else { 0 },
            peak_throughput: Self::calculate_peak_throughput(&v0_tape.frames),
        }
    }
}
```

#### Afternoon: Advanced Features & CLI Integration
```rust
// 5. Implement format utilities
pub struct TapeFormatUtils;

impl TapeFormatUtils {
    pub async fn repair_tape(tape_path: &Path) -> FormatResult<ValidationReport> {
        let mut report = ValidationReport::new();
        
        // Load and validate tape
        let tape_data = tokio::fs::read(tape_path).await?;
        let mut tape = TapeMigrator::migrate_to_latest(&tape_data)?;
        
        // Attempt repairs
        Self::repair_timestamps(&mut tape, &mut report)?;
        Self::repair_statistics(&mut tape, &mut report)?;
        Self::repair_index(&mut tape, &mut report)?;
        
        // Recalculate checksum
        tape.integrity.checksum = tape.calculate_checksum()?;
        
        // Save repaired tape
        let repaired_data = serde_json::to_vec_pretty(&tape)?;
        tokio::fs::write(tape_path, repaired_data).await?;
        
        report.add_info("Tape repair completed");
        Ok(report)
    }
    
    pub async fn analyze_tape(tape_path: &Path) -> FormatResult<TapeAnalysis> {
        let tape_data = tokio::fs::read(tape_path).await?;
        let tape = TapeMigrator::migrate_to_latest(&tape_data)?;
        
        let analysis = TapeAnalysis {
            format_version: tape.format.schema_version,
            health_score: Self::calculate_health_score(&tape),
            performance_metrics: Self::analyze_performance(&tape),
            recommendations: Self::generate_recommendations(&tape),
            compatibility: Self::check_compatibility(&tape),
        };
        
        Ok(analysis)
    }
    
    fn calculate_health_score(tape: &TapeFormatV1) -> f64 {
        let mut score = 100.0;
        
        // Deduct for validation issues
        match tape.validate() {
            Ok(report) => {
                score -= report.warnings.len() as f64 * 2.0;
            }
            Err(_) => {
                score -= 50.0; // Major deduction for validation failures
            }
        }
        
        // Deduct for missing metadata
        if tape.session.description.is_none() {
            score -= 5.0;
        }
        
        if tape.session.tags.is_empty() {
            score -= 3.0;
        }
        
        // Bonus for compression
        if matches!(tape.integrity.compression, CompressionType::Gzip | CompressionType::Zstd) {
            score += 5.0;
        }
        
        score.max(0.0).min(100.0)
    }
}

// 6. CLI integration for format operations
pub struct FormatCliCommands;

impl FormatCliCommands {
    pub async fn validate_tape(tape_id: &TapeId, recorder: &TapeRecorder) -> Result<()> {
        let tape = recorder.load_tape(tape_id).await?;
        
        println!("🔍 Validating tape: {}", tape.metadata.name);
        
        match tape.validate() {
            Ok(report) => {
                println!("✅ Validation passed");
                
                if !report.warnings.is_empty() {
                    println!("⚠️  Warnings:");
                    for warning in report.warnings {
                        println!("   • {}", warning);
                    }
                }
                
                if !report.info.is_empty() {
                    println!("ℹ️  Information:");
                    for info in report.info {
                        println!("   • {}", info);
                    }
                }
            }
            Err(FormatError::ValidationFailed(report)) => {
                println!("❌ Validation failed");
                
                for error in report.errors {
                    println!("   ❌ {}", error);
                }
                
                for warning in report.warnings {
                    println!("   ⚠️  {}", warning);
                }
                
                return Err(FormatError::ValidationFailed(report).into());
            }
            Err(e) => return Err(e.into()),
        }
        
        Ok(())
    }
    
    pub async fn migrate_tape(tape_id: &TapeId, recorder: &TapeRecorder) -> Result<()> {
        println!("🔄 Migrating tape to latest format: {}", tape_id);
        
        // Load original tape data
        let tape_path = recorder.get_tape_path(tape_id);
        let original_data = tokio::fs::read(&tape_path).await?;
        
        // Migrate to latest format
        let migrated_tape = TapeMigrator::migrate_to_latest(&original_data)?;
        
        // Save migrated tape
        let migrated_data = serde_json::to_vec_pretty(&migrated_tape)?;
        tokio::fs::write(&tape_path, migrated_data).await?;
        
        println!("✅ Migration completed successfully");
        println!("   Format version: {} → {}", 
                 TapeMigrator::detect_version(&original_data)?, 
                 migrated_tape.format.schema_version);
        
        Ok(())
    }
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_format_validation() {
        let tape = create_valid_v1_tape();
        let validation = tape.validate().unwrap();
        assert!(validation.errors.is_empty());
    }

    #[test]
    fn test_checksum_calculation() {
        let tape = create_test_v1_tape();
        let checksum = tape.calculate_checksum().unwrap();
        assert!(checksum.starts_with("sha256:"));
        assert_eq!(checksum.len(), 71); // "sha256:" + 64 hex chars
    }

    #[tokio::test]
    async fn test_compression() {
        let tape = create_large_test_tape();
        let compressed = tape.compress().unwrap();
        let decompressed = TapeFormatV1::decompress(&compressed).unwrap();
        
        assert_eq!(tape.frames.len(), decompressed.frames.len());
        assert!(compressed.len() < serde_json::to_vec(&tape).unwrap().len());
    }

    #[test]
    fn test_v0_to_v1_migration() {
        let v0_tape = create_v0_tape();
        let v0_data = serde_json::to_vec(&v0_tape).unwrap();
        
        let v1_tape = TapeMigrator::migrate_v0_to_v1(&v0_data).unwrap();
        
        assert_eq!(v1_tape.format.schema_version, 1);
        assert_eq!(v1_tape.frames.len(), v0_tape.frames.len());
        assert!(v1_tape.validate().is_ok());
    }

    #[test]
    fn test_format_detection() {
        let v0_data = create_v0_tape_data();
        let v1_data = create_v1_tape_data();
        
        assert_eq!(TapeMigrator::detect_version(&v0_data).unwrap(), 0);
        assert_eq!(TapeMigrator::detect_version(&v1_data).unwrap(), 1);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_format_workflow_integration() {
    let temp_dir = TempDir::new().unwrap();
    let recorder = TapeRecorder::new(temp_dir.path());
    
    // 1. Create V0 tape
    let session_id = SessionId::new();
    let session = Session::new(session_id.clone(), TransportType::Stdio);
    let tape_id = recorder.start_recording(&session, "test-migration".to_string()).await.unwrap();
    
    // Add some frames
    for i in 0..10 {
        let message = TransportMessage::new_request(
            i.to_string(),
            "test".to_string(),
            json!({"value": i}),
        );
        let frame = Frame::new(session_id.clone(), Direction::ClientToServer, message);
        recorder.record_frame(frame).await.unwrap();
    }
    
    let _tape = recorder.stop_recording(&session_id).await.unwrap();
    
    // 2. Migrate to V1
    FormatCliCommands::migrate_tape(&tape_id, &recorder).await.unwrap();
    
    // 3. Validate migrated tape
    FormatCliCommands::validate_tape(&tape_id, &recorder).await.unwrap();
    
    // 4. Load and verify V1 format
    let migrated_tape = recorder.load_tape(&tape_id).await.unwrap();
    // Verify V1 features are present
    assert!(migrated_tape.environment.is_some());
    assert!(migrated_tape.statistics.is_some());
    assert!(migrated_tape.index.is_some());
}
```

---

## Error Handling

### Error Types
```rust
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
    
    #[error("Unknown tape format - unable to detect version")]
    UnknownFormat,
    
    #[error("Validation failed: {0}")]
    ValidationFailed(ValidationReport),
    
    #[error("Migration failed from v{from} to v{to}: {reason}")]
    MigrationFailed { from: u32, to: u32, reason: String },
    
    #[error("Checksum verification failed - tape may be corrupted")]
    ChecksumMismatch,
    
    #[error("Compression error: {0}")]
    CompressionError(String),
    
    #[error("Decompression error: {0}")]
    DecompressionError(String),
}
```

---

## Success Criteria

### Functional Requirements  
- [x] V1 format includes comprehensive metadata and statistics
- [x] Migration from V0 to V1 preserves all essential data
- [x] Validation catches format inconsistencies and corruption
- [x] Compression reduces storage requirements significantly
- [x] CLI commands provide easy format management

### Quality Requirements
- [x] Migration is lossless and reversible 
- [x] Validation provides actionable error messages
- [x] Format is extensible for future enhancements
- [x] Performance impact is minimal for normal operations
- [x] Comprehensive test coverage for all format operations

### Performance Requirements
- [x] Migration completes within 10 seconds for 1000-frame tapes
- [x] Validation completes within 2 seconds for typical tapes  
- [x] Compression achieves 60%+ size reduction for JSON data
- [x] Format detection is reliable and fast

This enhanced tape format provides a robust foundation for advanced Shadowcat features while maintaining backward compatibility and operational simplicity.