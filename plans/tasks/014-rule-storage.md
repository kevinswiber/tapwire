# Task 014: Persistent Rule Storage

**Phase:** 4 - Interception & Rule Engine  
**Priority:** Medium  
**Estimated Effort:** 1 day  
**Assignee:** Development Team  
**Status:** Not Started

---

## Overview

Implement a comprehensive rule storage system that enables persistent rule management, versioning, collections, and hot-reloading. This system provides the foundation for rule libraries, sharing, and collaborative rule development while maintaining high performance for rule loading and updates.

## Objectives

- Create persistent storage for interception rules with versioning
- Implement rule collections and libraries for organization
- Add hot-reloading capabilities for development workflows
- Support import/export in multiple formats (JSON, YAML)
- Enable rule dependency tracking and conflict detection
- Provide rule usage analytics and performance metrics

## Technical Requirements

### Core Components

#### 1. Rule Storage System
```rust
pub struct RuleStorage {
    storage_dir: PathBuf,
    collections: RwLock<HashMap<String, RuleCollection>>,
    file_watcher: Option<RecommendedWatcher>,
    metrics: RuleStorageMetrics,
    config: RuleStorageConfig,
}

impl RuleStorage {
    pub fn new<P: AsRef<Path>>(storage_dir: P, config: RuleStorageConfig) -> Self;
    
    pub async fn initialize(&mut self) -> StorageResult<()>;
    pub async fn save_rule(&self, rule: &Rule) -> StorageResult<()>;
    pub async fn load_rule(&self, rule_id: &str) -> StorageResult<Option<Rule>>;
    pub async fn delete_rule(&self, rule_id: &str) -> StorageResult<()>;
    pub async fn list_rules(&self) -> StorageResult<Vec<RuleMetadata>>;
    pub async fn search_rules(&self, criteria: &SearchCriteria) -> StorageResult<Vec<RuleMetadata>>;
    
    pub async fn create_collection(&self, collection: &RuleCollection) -> StorageResult<()>;
    pub async fn load_collection(&self, collection_id: &str) -> StorageResult<Option<RuleCollection>>;
    pub async fn list_collections(&self) -> StorageResult<Vec<CollectionMetadata>>;
    
    pub async fn import_rules(&self, import_spec: &ImportSpec) -> StorageResult<ImportResult>;
    pub async fn export_rules(&self, export_spec: &ExportSpec) -> StorageResult<()>;
    
    pub async fn enable_hot_reload(&mut self) -> StorageResult<()>;
    pub async fn disable_hot_reload(&mut self) -> StorageResult<()>;
    
    pub fn get_metrics(&self) -> RuleStorageMetrics;
}

#[derive(Debug, Clone)]
pub struct RuleStorageConfig {
    pub auto_backup: bool,
    pub backup_retention_days: u32,
    pub enable_versioning: bool,
    pub max_versions_per_rule: u32,
    pub enable_hot_reload: bool,
    pub watch_subdirectories: bool,
    pub compression_enabled: bool,
    pub validation_on_load: bool,
}

impl Default for RuleStorageConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_retention_days: 30,
            enable_versioning: true,
            max_versions_per_rule: 10,
            enable_hot_reload: false,
            watch_subdirectories: true,
            compression_enabled: false,
            validation_on_load: true,
        }
    }
}
```

#### 2. Rule Collections
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
    pub rules: Vec<String>, // Rule IDs
    pub dependencies: Vec<CollectionDependency>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDependency {
    pub collection_id: String,
    pub version_requirement: String, // Semantic version requirement
    pub optional: bool,
}

impl RuleCollection {
    pub fn new(id: String, name: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id,
            name,
            description: None,
            version: "1.0.0".to_string(),
            author: None,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
            rules: Vec::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_rule(&mut self, rule_id: String) {
        if !self.rules.contains(&rule_id) {
            self.rules.push(rule_id);
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }
    
    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r == rule_id) {
            self.rules.remove(pos);
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            true
        } else {
            false
        }
    }
    
    pub fn validate_dependencies(&self, available_collections: &[CollectionMetadata]) -> Vec<String> {
        let mut missing_deps = Vec::new();
        
        for dep in &self.dependencies {
            let found = available_collections.iter()
                .find(|c| c.id == dep.collection_id)
                .map(|c| self.version_satisfies(&c.version, &dep.version_requirement))
                .unwrap_or(false);
                
            if !found && !dep.optional {
                missing_deps.push(format!("{}@{}", dep.collection_id, dep.version_requirement));
            }
        }
        
        missing_deps
    }
    
    fn version_satisfies(&self, version: &str, requirement: &str) -> bool {
        // Implement semantic version matching
        // For now, simple string comparison
        version == requirement || requirement == "*"
    }
}
```

#### 3. Rule Versioning
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleVersion {
    pub rule_id: String,
    pub version: u32,
    pub rule: Rule,
    pub created_at: u64,
    pub created_by: Option<String>,
    pub change_description: Option<String>,
    pub parent_version: Option<u32>,
    pub checksum: String,
}

impl RuleVersion {
    pub fn new(rule: Rule, version: u32, change_description: Option<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let rule_json = serde_json::to_string(&rule).unwrap_or_default();
        let checksum = format!("{:x}", sha2::Sha256::digest(rule_json.as_bytes()));
        
        Self {
            rule_id: rule.id.clone(),
            version,
            rule,
            created_at: now,
            created_by: None,
            change_description,
            parent_version: if version > 1 { Some(version - 1) } else { None },
            checksum,
        }
    }
}

pub struct RuleVersionManager {
    storage_dir: PathBuf,
    config: RuleStorageConfig,
}

impl RuleVersionManager {
    pub async fn save_version(&self, version: &RuleVersion) -> StorageResult<()>;
    pub async fn load_version(&self, rule_id: &str, version: u32) -> StorageResult<Option<RuleVersion>>;
    pub async fn list_versions(&self, rule_id: &str) -> StorageResult<Vec<RuleVersionMetadata>>;
    pub async fn get_latest_version(&self, rule_id: &str) -> StorageResult<Option<RuleVersion>>;
    pub async fn cleanup_old_versions(&self, rule_id: &str) -> StorageResult<u32>;
    
    pub async fn diff_versions(
        &self,
        rule_id: &str,
        version1: u32,
        version2: u32,
    ) -> StorageResult<RuleVersionDiff>;
}

#[derive(Debug, Clone)]
pub struct RuleVersionDiff {
    pub rule_id: String,
    pub version1: u32,
    pub version2: u32,
    pub changes: Vec<RuleChange>,
}

#[derive(Debug, Clone)]
pub enum RuleChange {
    NameChanged { from: String, to: String },
    DescriptionChanged { from: Option<String>, to: Option<String> },
    PriorityChanged { from: u32, to: u32 },
    EnabledChanged { from: bool, to: bool },
    ConditionsChanged { diff: String },
    ActionsChanged { diff: String },
    MetadataChanged { key: String, from: Option<serde_json::Value>, to: Option<serde_json::Value> },
}
```

#### 4. Hot Reloading
```rust
pub struct HotReloader {
    watcher: RecommendedWatcher,
    event_tx: mpsc::UnboundedSender<ReloadEvent>,
    rule_engine: Arc<RuleEngine>,
    storage: Arc<RuleStorage>,
    config: HotReloadConfig,
}

impl HotReloader {
    pub fn new(
        rule_engine: Arc<RuleEngine>,
        storage: Arc<RuleStorage>,
        config: HotReloadConfig,
    ) -> StorageResult<Self>;
    
    pub async fn start(&mut self) -> StorageResult<()>;
    pub async fn stop(&mut self) -> StorageResult<()>;
    
    async fn handle_file_event(&self, event: notify::Event) -> StorageResult<()> {
        match event.kind {
            notify::EventKind::Create(_) => {
                for path in event.paths {
                    if self.is_rule_file(&path) {
                        self.reload_rule_file(&path).await?;
                    }
                }
            }
            notify::EventKind::Modify(_) => {
                for path in event.paths {
                    if self.is_rule_file(&path) {
                        self.reload_rule_file(&path).await?;
                    }
                }
            }
            notify::EventKind::Remove(_) => {
                for path in event.paths {
                    if self.is_rule_file(&path) {
                        self.remove_rule_from_engine(&path).await?;
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    async fn reload_rule_file(&self, path: &Path) -> StorageResult<()> {
        // Debounce rapid file changes
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        match self.load_and_validate_rule(path).await {
            Ok(rule) => {
                self.rule_engine.add_or_update_rule(rule).await?;
                info!("Hot-reloaded rule from: {}", path.display());
            }
            Err(e) => {
                warn!("Failed to hot-reload rule from {}: {}", path.display(), e);
            }
        }
        
        Ok(())
    }
    
    fn is_rule_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "json" || ext == "yaml" || ext == "yml")
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    pub debounce_ms: u64,
    pub watch_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub validate_on_reload: bool,
    pub notify_on_error: bool,
}

#[derive(Debug, Clone)]
pub enum ReloadEvent {
    RuleAdded(String),
    RuleUpdated(String),
    RuleRemoved(String),
    CollectionUpdated(String),
    ReloadError(String, String),
}
```

#### 5. Import/Export System
```rust
#[derive(Debug, Clone)]
pub struct ImportSpec {
    pub source: ImportSource,
    pub target_collection: Option<String>,
    pub overwrite_existing: bool,
    pub validate_rules: bool,
    pub create_backup: bool,
}

#[derive(Debug, Clone)]
pub enum ImportSource {
    File(PathBuf),
    Directory(PathBuf),
    Url(String),
    Archive(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ExportSpec {
    pub rules: Vec<String>, // Rule IDs to export, empty = all
    pub collections: Vec<String>, // Collection IDs to export
    pub format: ExportFormat,
    pub destination: PathBuf,
    pub include_metadata: bool,
    pub include_versions: bool,
    pub compress: bool,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Yaml,
    Archive, // ZIP with rules and collections
    Bundle,  // Single file with everything
}

impl RuleStorage {
    pub async fn import_rules(&self, spec: &ImportSpec) -> StorageResult<ImportResult> {
        let mut result = ImportResult::default();
        
        match &spec.source {
            ImportSource::File(path) => {
                result = self.import_from_file(path, spec).await?;
            }
            ImportSource::Directory(path) => {
                result = self.import_from_directory(path, spec).await?;
            }
            ImportSource::Url(url) => {
                result = self.import_from_url(url, spec).await?;
            }
            ImportSource::Archive(path) => {
                result = self.import_from_archive(path, spec).await?;
            }
        }
        
        // Update metrics
        self.metrics.imports_total.fetch_add(1, Ordering::Relaxed);
        self.metrics.imported_rules.fetch_add(result.imported_rules.len(), Ordering::Relaxed);
        
        Ok(result)
    }
    
    async fn import_from_file(&self, path: &Path, spec: &ImportSpec) -> StorageResult<ImportResult> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
            
        let rules: Vec<Rule> = if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            serde_yaml::from_str(&content)
                .map_err(|e| StorageError::ParseError(e.to_string()))?
        } else {
            serde_json::from_str(&content)
                .map_err(|e| StorageError::ParseError(e.to_string()))?
        };
        
        let mut result = ImportResult::default();
        
        for rule in rules {
            match self.import_single_rule(rule, spec).await {
                Ok(rule_id) => {
                    result.imported_rules.push(rule_id);
                }
                Err(e) => {
                    result.failed_rules.push(ImportError {
                        rule_id: rule.id.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
        
        Ok(result)
    }
    
    pub async fn export_rules(&self, spec: &ExportSpec) -> StorageResult<()> {
        let rules = self.collect_rules_for_export(spec).await?;
        let collections = self.collect_collections_for_export(spec).await?;
        
        match spec.format {
            ExportFormat::Json => {
                self.export_as_json(&rules, &collections, spec).await?;
            }
            ExportFormat::Yaml => {
                self.export_as_yaml(&rules, &collections, spec).await?;
            }
            ExportFormat::Archive => {
                self.export_as_archive(&rules, &collections, spec).await?;
            }
            ExportFormat::Bundle => {
                self.export_as_bundle(&rules, &collections, spec).await?;
            }
        }
        
        // Update metrics
        self.metrics.exports_total.fetch_add(1, Ordering::Relaxed);
        self.metrics.exported_rules.fetch_add(rules.len(), Ordering::Relaxed);
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ImportResult {
    pub imported_rules: Vec<String>,
    pub imported_collections: Vec<String>,
    pub failed_rules: Vec<ImportError>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct ImportError {
    pub rule_id: String,
    pub error: String,
}
```

#### 6. Storage Metrics and Analytics
```rust
#[derive(Debug, Default)]
pub struct RuleStorageMetrics {
    pub total_rules: AtomicUsize,
    pub total_collections: AtomicUsize,
    pub total_versions: AtomicUsize,
    pub imports_total: AtomicUsize,
    pub exports_total: AtomicUsize,
    pub imported_rules: AtomicUsize,
    pub exported_rules: AtomicUsize,
    pub hot_reloads: AtomicUsize,
    pub validation_errors: AtomicUsize,
    pub storage_size_bytes: AtomicU64,
    pub rule_usage_stats: RwLock<HashMap<String, RuleUsageStats>>,
}

#[derive(Debug, Clone)]
pub struct RuleUsageStats {
    pub rule_id: String,
    pub execution_count: u64,
    pub last_executed: Option<u64>,
    pub average_execution_time_us: u64,
    pub success_rate: f32,
    pub error_count: u64,
}

impl RuleStorageMetrics {
    pub fn record_rule_execution(&self, rule_id: &str, execution_time_us: u64, success: bool) {
        let mut stats = self.rule_usage_stats.write().unwrap();
        let entry = stats.entry(rule_id.to_string()).or_insert_with(|| {
            RuleUsageStats {
                rule_id: rule_id.to_string(),
                execution_count: 0,
                last_executed: None,
                average_execution_time_us: 0,
                success_rate: 1.0,
                error_count: 0,
            }
        });
        
        entry.execution_count += 1;
        entry.last_executed = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
            
        // Update average execution time
        entry.average_execution_time_us = 
            (entry.average_execution_time_us * (entry.execution_count - 1) + execution_time_us) 
            / entry.execution_count;
            
        // Update success rate
        if !success {
            entry.error_count += 1;
        }
        entry.success_rate = 
            (entry.execution_count - entry.error_count) as f32 / entry.execution_count as f32;
    }
    
    pub fn get_top_rules_by_usage(&self, limit: usize) -> Vec<RuleUsageStats> {
        let stats = self.rule_usage_stats.read().unwrap();
        let mut sorted_stats: Vec<RuleUsageStats> = stats.values().cloned().collect();
        sorted_stats.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        sorted_stats.truncate(limit);
        sorted_stats
    }
}
```

## Implementation Details

### Phase 1: Basic Storage (Day 1)
1. Implement RuleStorage with file-based persistence
2. Add rule CRUD operations with JSON/YAML support
3. Create rule versioning system
4. Add basic collection management
5. Write unit tests for storage operations

### Phase 2: Advanced Features (Day 1)
1. Implement hot-reloading with file watching
2. Add import/export functionality
3. Create dependency tracking and validation
4. Add storage metrics and analytics
5. Integration testing with rule engine

## Acceptance Criteria

### Functional Requirements
- [ ] Rules persist correctly across application restarts
- [ ] Rule versioning tracks changes and allows rollback
- [ ] Collections organize rules with dependency management
- [ ] Hot-reloading updates rules without restart
- [ ] Import/export works with JSON and YAML formats
- [ ] Storage metrics track usage and performance

### Performance Requirements
- [ ] Rule loading completes in < 100ms for 1000 rules
- [ ] Hot-reload detects changes within 1 second
- [ ] Storage operations scale linearly with rule count
- [ ] Memory usage stays under 10MB for typical rule sets

### Quality Requirements
- [ ] Data integrity maintained across all operations
- [ ] Concurrent access is thread-safe
- [ ] Comprehensive error handling and recovery
- [ ] Backup and restore functionality works correctly

## Dependencies

### Internal Dependencies
- Rule and RuleEngine from rule engine (Task 011)
- Error handling framework

### External Dependencies
- `notify` crate for file system watching
- `serde_yaml` for YAML support
- `zip` crate for archive support
- `sha2` crate for checksums

## Definition of Done

- [ ] All acceptance criteria met
- [ ] Tests passing with > 95% coverage
- [ ] Hot-reloading works reliably
- [ ] Import/export handles edge cases
- [ ] Documentation complete
- [ ] Integration with rule engine working

## Follow-up Tasks

- Rule marketplace and sharing platform
- Advanced rule analytics and recommendations
- Rule A/B testing framework
- Integration with version control systems