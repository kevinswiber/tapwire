# Task 009: Storage Optimization

**Files:** `src/recorder/storage.rs`, `src/recorder/index.rs`  
**Estimated Effort:** 1.5 days  
**Priority:** Medium  
**Dependencies:** Enhanced Tape Format, TapeRecorder

---

## Overview

Optimize Shadowcat's tape storage system with indexing, search capabilities, cleanup policies, and performance enhancements to efficiently manage large collections of recorded MCP sessions.

---

## Requirements

### Core Optimization Features
1. **Tape Indexing**: Fast metadata access and search capabilities
2. **Storage Policies**: Configurable cleanup rules (TTL, size limits, count limits)
3. **Search & Filtering**: Query tapes by various criteria
4. **Statistics & Analytics**: Aggregated insights across tape collections
5. **Background Maintenance**: Automated cleanup and optimization tasks

### Performance Enhancements
1. **Lazy Loading**: Load tape content on-demand
2. **Metadata Caching**: In-memory cache for frequently accessed data
3. **Concurrent Access**: Thread-safe operations with minimal locking
4. **Streaming Operations**: Handle large tapes without full memory load
5. **Compression Management**: Automatic compression of old tapes

---

## Technical Specification

### Storage Index System
```rust
// src/recorder/index.rs
use crate::error::{StorageError, StorageResult};
use crate::recorder::{TapeId, TapeMetadata};
use crate::transport::TransportType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeIndex {
    /// Index version for migration compatibility
    pub version: u32,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Map of tape ID to index entry
    pub entries: HashMap<TapeId, IndexEntry>,
    /// Timeline index sorted by creation time
    pub timeline: BTreeMap<DateTime<Utc>, TapeId>,
    /// Index by transport type
    pub by_transport: HashMap<TransportType, Vec<TapeId>>,
    /// Index by tags
    pub by_tags: HashMap<String, Vec<TapeId>>,
    /// Storage statistics
    pub statistics: IndexStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub tape_id: TapeId,
    pub file_path: PathBuf,
    pub metadata: TapeIndexMetadata,
    pub file_info: FileInfo,
    pub last_accessed: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapeIndexMetadata {
    pub name: String,
    pub description: Option<String>,
    pub transport_type: TransportType,
    pub created_at: DateTime<Utc>,
    pub duration_ms: Option<u64>,
    pub frame_count: usize,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub size_bytes: u64,
    pub compressed: bool,
    pub checksum: String,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_tapes: usize,
    pub total_size_bytes: u64,
    pub total_frames: usize,
    pub compression_ratio: f64,
    pub by_transport: HashMap<TransportType, TransportStats>,
    pub oldest_tape: Option<DateTime<Utc>>,
    pub newest_tape: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStats {
    pub count: usize,
    pub total_frames: usize,
    pub avg_duration_ms: f64,
    pub total_size_bytes: u64,
}

pub struct TapeIndexManager {
    index_path: PathBuf,
    index: RwLock<TapeIndex>,
    cache_size: usize,
    metadata_cache: RwLock<lru::LruCache<TapeId, TapeIndexMetadata>>,
}

impl TapeIndexManager {
    pub async fn new<P: AsRef<Path>>(storage_dir: P) -> StorageResult<Self> {
        let index_path = storage_dir.as_ref().join("index.json");
        let cache_size = 1000; // Cache up to 1000 tape metadata entries
        
        let index = if index_path.exists() {
            Self::load_index(&index_path).await?
        } else {
            TapeIndex::new()
        };
        
        Ok(Self {
            index_path,
            index: RwLock::new(index),
            cache_size,
            metadata_cache: RwLock::new(lru::LruCache::new(cache_size)),
        })
    }
    
    pub async fn add_tape(&self, tape_id: TapeId, file_path: PathBuf, metadata: TapeIndexMetadata) -> StorageResult<()> {
        let file_info = Self::collect_file_info(&file_path).await?;
        
        let entry = IndexEntry {
            tape_id: tape_id.clone(),
            file_path,
            metadata: metadata.clone(),
            file_info,
            last_accessed: None,
        };
        
        {
            let mut index = self.index.write().await;
            
            // Add to main entries
            index.entries.insert(tape_id.clone(), entry);
            
            // Update timeline index
            index.timeline.insert(metadata.created_at, tape_id.clone());
            
            // Update transport index
            index.by_transport
                .entry(metadata.transport_type)
                .or_insert_with(Vec::new)
                .push(tape_id.clone());
            
            // Update tag index
            for tag in &metadata.tags {
                index.by_tags
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(tape_id.clone());
            }
            
            // Update statistics
            Self::update_statistics(&mut index.statistics, &metadata, &entry.file_info, true);
            
            index.updated_at = Utc::now();
        }
        
        // Cache metadata
        {
            let mut cache = self.metadata_cache.write().await;
            cache.put(tape_id, metadata);
        }
        
        // Persist index
        self.save_index().await?;
        
        Ok(())
    }
    
    pub async fn remove_tape(&self, tape_id: &TapeId) -> StorageResult<()> {
        let entry = {
            let mut index = self.index.write().await;
            
            let entry = index.entries.remove(tape_id)
                .ok_or_else(|| StorageError::NotFound(tape_id.to_string()))?;
            
            // Remove from timeline
            index.timeline.remove(&entry.metadata.created_at);
            
            // Remove from transport index
            if let Some(transport_list) = index.by_transport.get_mut(&entry.metadata.transport_type) {
                transport_list.retain(|id| id != tape_id);
            }
            
            // Remove from tag indexes
            for tag in &entry.metadata.tags {
                if let Some(tag_list) = index.by_tags.get_mut(tag) {
                    tag_list.retain(|id| id != tape_id);
                }
            }
            
            // Update statistics
            Self::update_statistics(&mut index.statistics, &entry.metadata, &entry.file_info, false);
            
            index.updated_at = Utc::now();
            
            entry
        };
        
        // Remove from cache
        {
            let mut cache = self.metadata_cache.write().await;
            cache.pop(tape_id);
        }
        
        // Persist index
        self.save_index().await?;
        
        Ok(())
    }
    
    pub async fn search(&self, query: &SearchQuery) -> StorageResult<Vec<SearchResult>> {
        let index = self.index.read().await;
        let mut results = Vec::new();
        
        for (tape_id, entry) in &index.entries {
            if Self::matches_query(&entry.metadata, query) {
                let score = Self::calculate_relevance_score(&entry.metadata, query);
                results.push(SearchResult {
                    tape_id: tape_id.clone(),
                    metadata: entry.metadata.clone(),
                    file_info: entry.file_info.clone(),
                    relevance_score: score,
                });
            }
        }
        
        // Sort by relevance score
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        
        Ok(results)
    }
    
    pub async fn get_statistics(&self) -> StorageResult<IndexStatistics> {
        let index = self.index.read().await;
        Ok(index.statistics.clone())
    }
}
```

### Storage Policy System
```rust
// src/recorder/storage.rs
use crate::error::{StorageError, StorageResult};
use crate::recorder::{TapeId, TapeIndexManager};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, instrument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePolicy {
    /// Maximum age for tapes before cleanup
    pub max_age_days: Option<u32>,
    /// Maximum total storage size in bytes
    pub max_total_size_bytes: Option<u64>,
    /// Maximum number of tapes to keep
    pub max_tape_count: Option<usize>,
    /// Minimum free disk space to maintain (bytes)
    pub min_free_space_bytes: Option<u64>,
    /// Automatically compress tapes older than N days
    pub auto_compress_after_days: Option<u32>,
    /// Delete tapes not accessed for N days
    pub delete_unused_after_days: Option<u32>,
    /// Preserve tapes with specific tags
    pub protected_tags: Vec<String>,
}

impl Default for StoragePolicy {
    fn default() -> Self {
        Self {
            max_age_days: Some(90),           // 90 days
            max_total_size_bytes: Some(10 * 1024 * 1024 * 1024), // 10 GB
            max_tape_count: Some(10_000),     // 10K tapes
            min_free_space_bytes: Some(1024 * 1024 * 1024), // 1 GB
            auto_compress_after_days: Some(7), // 1 week
            delete_unused_after_days: Some(30), // 30 days
            protected_tags: vec!["important".to_string(), "production".to_string()],
        }
    }
}

pub struct StorageManager {
    storage_dir: PathBuf,
    index_manager: TapeIndexManager,
    policy: StoragePolicy,
    maintenance_running: std::sync::atomic::AtomicBool,
}

impl StorageManager {
    pub async fn new<P: AsRef<Path>>(storage_dir: P, policy: StoragePolicy) -> StorageResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        let index_manager = TapeIndexManager::new(&storage_dir).await?;
        
        Ok(Self {
            storage_dir,
            index_manager,
            policy,
            maintenance_running: std::sync::atomic::AtomicBool::new(false),
        })
    }
    
    #[instrument(skip(self))]
    pub async fn apply_cleanup_policies(&self) -> StorageResult<CleanupReport> {
        info!("Starting storage cleanup with policies");
        
        let mut report = CleanupReport::new();
        
        // Get current statistics
        let stats = self.index_manager.get_statistics().await?;
        
        // Apply age-based cleanup
        if let Some(max_age_days) = self.policy.max_age_days {
            self.cleanup_by_age(max_age_days, &mut report).await?;
        }
        
        // Apply size-based cleanup
        if let Some(max_size) = self.policy.max_total_size_bytes {
            if stats.total_size_bytes > max_size {
                self.cleanup_by_size(max_size, &mut report).await?;
            }
        }
        
        // Apply count-based cleanup
        if let Some(max_count) = self.policy.max_tape_count {
            if stats.total_tapes > max_count {
                self.cleanup_by_count(max_count, &mut report).await?;
            }
        }
        
        // Apply unused tape cleanup
        if let Some(unused_days) = self.policy.delete_unused_after_days {
            self.cleanup_unused_tapes(unused_days, &mut report).await?;
        }
        
        // Auto-compress old tapes
        if let Some(compress_days) = self.policy.auto_compress_after_days {
            self.auto_compress_tapes(compress_days, &mut report).await?;
        }
        
        info!("Storage cleanup completed: {}", report.summary());
        Ok(report)
    }
    
    async fn cleanup_by_age(&self, max_age_days: u32, report: &mut CleanupReport) -> StorageResult<()> {
        let cutoff_date = Utc::now() - Duration::days(max_age_days as i64);
        
        let query = SearchQuery {
            created_before: Some(cutoff_date),
            exclude_tags: self.policy.protected_tags.clone(),
            ..Default::default()
        };
        
        let old_tapes = self.index_manager.search(&query).await?;
        
        for result in old_tapes {
            if !self.is_protected(&result.metadata) {
                self.delete_tape(&result.tape_id).await?;
                report.deleted_tapes.push(result.tape_id);
                report.bytes_freed += result.file_info.size_bytes;
            }
        }
        
        Ok(())
    }
    
    async fn auto_compress_tapes(&self, compress_days: u32, report: &mut CleanupReport) -> StorageResult<()> {
        let cutoff_date = Utc::now() - Duration::days(compress_days as i64);
        
        let query = SearchQuery {
            created_before: Some(cutoff_date),
            compressed: Some(false), // Only uncompressed tapes
            ..Default::default()
        };
        
        let uncompressed_tapes = self.index_manager.search(&query).await?;
        
        for result in uncompressed_tapes {
            match self.compress_tape(&result.tape_id).await {
                Ok(compression_ratio) => {
                    report.compressed_tapes.push(result.tape_id);
                    report.bytes_saved_compression += 
                        (result.file_info.size_bytes as f64 * (1.0 - compression_ratio)) as u64;
                }
                Err(e) => {
                    warn!("Failed to compress tape {}: {}", result.tape_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    pub async fn start_background_maintenance(&self) -> StorageResult<()> {
        use std::sync::atomic::Ordering;
        
        if self.maintenance_running.swap(true, Ordering::SeqCst) {
            return Ok(());  // Already running
        }
        
        let storage_manager = self.clone(); // Implement Clone for StorageManager
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::hours(1).to_std().unwrap());
            
            loop {
                interval.tick().await;
                
                if let Err(e) = storage_manager.apply_cleanup_policies().await {
                    warn!("Background maintenance failed: {}", e);
                }
                
                // Check if we should stop
                if !storage_manager.maintenance_running.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CleanupReport {
    pub deleted_tapes: Vec<TapeId>,
    pub compressed_tapes: Vec<TapeId>,
    pub bytes_freed: u64,
    pub bytes_saved_compression: u64,
    pub errors: Vec<String>,
}

impl CleanupReport {
    pub fn new() -> Self {
        Self {
            deleted_tapes: Vec::new(),
            compressed_tapes: Vec::new(),
            bytes_freed: 0,
            bytes_saved_compression: 0,
            errors: Vec::new(),
        }
    }
    
    pub fn summary(&self) -> String {
        format!(
            "Deleted {} tapes ({} bytes), compressed {} tapes ({} bytes saved)",
            self.deleted_tapes.len(),
            self.bytes_freed,
            self.compressed_tapes.len(),
            self.bytes_saved_compression
        )
    }
}
```

### Search and Query System
```rust
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub name_contains: Option<String>,
    pub description_contains: Option<String>,
    pub transport_type: Option<TransportType>,
    pub tags_any: Vec<String>,
    pub tags_all: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub min_duration_ms: Option<u64>,
    pub max_duration_ms: Option<u64>,
    pub min_frame_count: Option<usize>,
    pub max_frame_count: Option<usize>,
    pub compressed: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub tape_id: TapeId,
    pub metadata: TapeIndexMetadata,
    pub file_info: FileInfo,
    pub relevance_score: f64,
}

impl TapeIndexManager {
    fn matches_query(metadata: &TapeIndexMetadata, query: &SearchQuery) -> bool {
        // Name filtering
        if let Some(name_filter) = &query.name_contains {
            if !metadata.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                return false;
            }
        }
        
        // Description filtering
        if let Some(desc_filter) = &query.description_contains {
            match &metadata.description {
                Some(desc) => {
                    if !desc.to_lowercase().contains(&desc_filter.to_lowercase()) {
                        return false;
                    }
                }
                None => return false,
            }
        }
        
        // Transport type filtering
        if let Some(transport) = &query.transport_type {
            if metadata.transport_type != *transport {
                return false;
            }
        }
        
        // Tag filtering
        if !query.tags_any.is_empty() {
            if !query.tags_any.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        if !query.tags_all.is_empty() {
            if !query.tags_all.iter().all(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        if !query.exclude_tags.is_empty() {
            if query.exclude_tags.iter().any(|tag| metadata.tags.contains(tag)) {
                return false;
            }
        }
        
        // Date range filtering
        if let Some(after) = query.created_after {
            if metadata.created_at <= after {
                return false;
            }
        }
        
        if let Some(before) = query.created_before {
            if metadata.created_at >= before {
                return false;
            }
        }
        
        // Duration filtering
        if let Some(duration) = metadata.duration_ms {
            if let Some(min_duration) = query.min_duration_ms {
                if duration < min_duration {
                    return false;
                }
            }
            
            if let Some(max_duration) = query.max_duration_ms {
                if duration > max_duration {
                    return false;
                }
            }
        }
        
        // Frame count filtering
        if let Some(min_frames) = query.min_frame_count {
            if metadata.frame_count < min_frames {
                return false;
            }
        }
        
        if let Some(max_frames) = query.max_frame_count {
            if metadata.frame_count > max_frames {
                return false;
            }
        }
        
        true
    }
    
    fn calculate_relevance_score(metadata: &TapeIndexMetadata, query: &SearchQuery) -> f64 {
        let mut score = 0.0;
        
        // Boost score for exact name matches
        if let Some(name_filter) = &query.name_contains {
            if metadata.name.to_lowercase() == name_filter.to_lowercase() {
                score += 10.0;
            } else if metadata.name.to_lowercase().contains(&name_filter.to_lowercase()) {
                score += 5.0;
            }
        }
        
        // Boost score for tag matches
        let matching_tags = query.tags_any.iter()
            .filter(|tag| metadata.tags.contains(tag))
            .count();
        score += matching_tags as f64 * 2.0;
        
        // Boost score for recent tapes
        let age_days = (Utc::now() - metadata.created_at).num_days();
        if age_days <= 7 {
            score += 3.0;
        } else if age_days <= 30 {
            score += 1.0;
        }
        
        // Boost score for larger tapes (more content)
        if metadata.frame_count > 100 {
            score += 1.0;
        }
        
        score
    }
}
```

---

## Implementation Plan

### Day 1: Indexing System

#### Morning: Index Structure & Manager
```rust
// 1. Implement TapeIndex and IndexEntry structures
// 2. Create TapeIndexManager with basic CRUD operations
// 3. Add index persistence (save/load from JSON)
// 4. Implement metadata caching with LRU cache
```

#### Afternoon: Search & Query System
```rust
// 5. Implement SearchQuery and SearchResult structures
// 6. Add search functionality with relevance scoring
// 7. Create query matching logic for all filter types
// 8. Add sorting and pagination support
```

### Day 2: Storage Policies & Maintenance

#### Morning: Storage Policies
```rust
// 9. Implement StoragePolicy configuration
// 10. Create StorageManager with policy enforcement
// 11. Add cleanup operations (age, size, count-based)
// 12. Implement tape compression automation
```

#### Afternoon: Background Maintenance & Integration
```rust
// 13. Add background maintenance task system
// 14. Implement cleanup reporting and logging
// 15. Integrate with existing TapeRecorder
// 16. Add CLI commands for storage management
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_creation_and_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let index_manager = TapeIndexManager::new(temp_dir.path()).await.unwrap();
        
        // Add test tape
        let tape_id = TapeId::new();
        let metadata = create_test_metadata();
        let file_path = temp_dir.path().join("test.json");
        
        index_manager.add_tape(tape_id.clone(), file_path, metadata.clone()).await.unwrap();
        
        // Verify indexing
        let search_results = index_manager.search(&SearchQuery::default()).await.unwrap();
        assert_eq!(search_results.len(), 1);
        assert_eq!(search_results[0].tape_id, tape_id);
    }

    #[tokio::test]
    async fn test_search_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let index_manager = TapeIndexManager::new(temp_dir.path()).await.unwrap();
        
        // Add multiple test tapes with different characteristics
        add_test_tapes(&index_manager).await;
        
        // Test name search
        let query = SearchQuery {
            name_contains: Some("test".to_string()),
            ..Default::default()
        };
        let results = index_manager.search(&query).await.unwrap();
        assert!(!results.is_empty());
        
        // Test tag search
        let query = SearchQuery {
            tags_any: vec!["debug".to_string()],
            ..Default::default()
        };
        let results = index_manager.search(&query).await.unwrap();
        // Verify results have debug tag
        
        // Test date range search
        let query = SearchQuery {
            created_after: Some(Utc::now() - chrono::Duration::hours(1)),
            ..Default::default()
        };
        let results = index_manager.search(&query).await.unwrap();
        // Verify results are recent
    }

    #[tokio::test]
    async fn test_storage_policy_enforcement() {
        let temp_dir = TempDir::new().unwrap();
        
        let policy = StoragePolicy {
            max_tape_count: Some(3),
            max_age_days: Some(1),
            ..Default::default()
        };
        
        let storage_manager = StorageManager::new(temp_dir.path(), policy).await.unwrap();
        
        // Add more tapes than policy allows
        add_many_test_tapes(&storage_manager, 5).await;
        
        // Apply cleanup policies
        let report = storage_manager.apply_cleanup_policies().await.unwrap();
        
        // Verify cleanup occurred
        assert!(!report.deleted_tapes.is_empty());
        assert!(report.bytes_freed > 0);
    }
}
```

### Performance Tests
```rust
#[tokio::test]
async fn test_large_collection_performance() {
    let temp_dir = TempDir::new().unwrap();
    let index_manager = TapeIndexManager::new(temp_dir.path()).await.unwrap();
    
    // Add 1000 tapes
    for i in 0..1000 {
        let metadata = create_test_metadata_with_index(i);
        let file_path = temp_dir.path().join(format!("tape_{}.json", i));
        index_manager.add_tape(TapeId::new(), file_path, metadata).await.unwrap();
    }
    
    // Test search performance
    let start = std::time::Instant::now();
    let results = index_manager.search(&SearchQuery::default()).await.unwrap();
    let duration = start.elapsed();
    
    assert_eq!(results.len(), 1000);
    assert!(duration.as_millis() < 100); // Should complete within 100ms
}
```

---

## CLI Integration

### Storage Management Commands
```rust
// Add to src/cli/tape.rs
#[derive(Args)]
pub struct StorageCommand {
    #[command(subcommand)]
    pub command: StorageSubcommand,
}

#[derive(Subcommand)]
pub enum StorageSubcommand {
    Status,
    Cleanup(CleanupArgs),
    Policy(PolicyArgs),
    Search(SearchArgs),
    Index(IndexArgs),
}

#[derive(Args)]
pub struct CleanupArgs {
    /// Apply cleanup policies now
    #[arg(long)]
    now: bool,
    
    /// Dry run - show what would be cleaned up
    #[arg(long)]
    dry_run: bool,
    
    /// Force cleanup ignoring protected tags
    #[arg(long)]
    force: bool,
}

impl StorageCommand {
    pub async fn execute(&self, storage_manager: &StorageManager) -> Result<()> {
        match &self.command {
            StorageSubcommand::Status => self.show_storage_status(storage_manager).await,
            StorageSubcommand::Cleanup(args) => self.run_cleanup(args, storage_manager).await,
            StorageSubcommand::Search(args) => self.search_tapes(args, storage_manager).await,
            // ... other commands
        }
    }
    
    async fn show_storage_status(&self, storage_manager: &StorageManager) -> Result<()> {
        let stats = storage_manager.get_statistics().await?;
        
        println!("📊 Storage Status");
        println!("   Total tapes: {}", stats.total_tapes);
        println!("   Total size: {}", format_bytes(stats.total_size_bytes));
        println!("   Total frames: {}", stats.total_frames);
        println!("   Compression ratio: {:.1}%", stats.compression_ratio * 100.0);
        
        if let Some(oldest) = stats.oldest_tape {
            println!("   Oldest tape: {}", oldest.format("%Y-%m-%d"));
        }
        
        if let Some(newest) = stats.newest_tape {
            println!("   Newest tape: {}", newest.format("%Y-%m-%d"));
        }
        
        // Show by transport type
        println!("\n📈 By Transport Type:");
        for (transport, transport_stats) in &stats.by_transport {
            println!("   {:?}: {} tapes, {} frames, {}", 
                     transport, 
                     transport_stats.count,
                     transport_stats.total_frames,
                     format_bytes(transport_stats.total_size_bytes));
        }
        
        Ok(())
    }
}
```

---

## Success Criteria

### Performance Requirements
- [x] Index operations complete within 50ms for 1000 tapes
- [x] Search queries return results within 100ms for large collections
- [x] Memory usage under 100MB for typical operations
- [x] Background maintenance has minimal performance impact

### Functional Requirements
- [x] Fast tape discovery and metadata access through indexing
- [x] Flexible search with multiple filter criteria
- [x] Automated cleanup based on configurable policies
- [x] Statistics and analytics for storage insights
- [x] Background maintenance with error handling

### Quality Requirements
- [x] Comprehensive test coverage for all storage operations
- [x] Thread-safe concurrent access to index and storage
- [x] Graceful error handling and recovery
- [x] Clear CLI interface for storage management

This storage optimization system provides the foundation for managing large-scale MCP session recording collections efficiently, with powerful search capabilities and automated maintenance.