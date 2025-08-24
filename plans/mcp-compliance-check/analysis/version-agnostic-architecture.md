# Version-Agnostic Compliance Architecture

## Design Principles

To support easy addition of new MCP versions (released every few months), our architecture must be:

1. **Modular** - Each version is a self-contained module
2. **Declarative** - Version differences expressed as configuration, not code
3. **Incremental** - New versions inherit from previous with overrides
4. **Discoverable** - Tests auto-detect applicable versions
5. **Maintainable** - Single source of truth for version features

## Core Architecture

### Version Registry Pattern

```rust
// Core trait that all versions implement
pub trait ProtocolVersion: Send + Sync {
    fn version_string(&self) -> &'static str;
    fn supports_feature(&self, feature: Feature) -> bool;
    fn capability_format(&self) -> CapabilityFormat;
    fn create_adapter(&self) -> Box<dyn ProtocolAdapter>;
    fn parent_version(&self) -> Option<Box<dyn ProtocolVersion>>;
}

// Feature flags for conditional behavior
#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    AsyncTools,
    BatchRequests,
    StructuredOutput,
    Elicitation,
    StreamableHttp,
    ObjectCapabilities,
    ResourceMetadata,
    ToolCancellation,
}

// Registry for all supported versions
pub struct VersionRegistry {
    versions: HashMap<String, Box<dyn ProtocolVersion>>,
}

impl VersionRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            versions: HashMap::new(),
        };
        
        // Register supported versions
        registry.register(Box::new(V2025_03_26));
        registry.register(Box::new(V2025_06_18));
        // Future: registry.register(Box::new(V2025_09_XX));
        
        registry
    }
    
    pub fn register(&mut self, version: Box<dyn ProtocolVersion>) {
        self.versions.insert(version.version_string().to_string(), version);
    }
    
    pub fn get(&self, version: &str) -> Option<&dyn ProtocolVersion> {
        self.versions.get(version).map(|v| v.as_ref())
    }
}
```

### Version Implementation Pattern

```rust
// Each version is a simple struct
pub struct V2025_03_26;

impl ProtocolVersion for V2025_03_26 {
    fn version_string(&self) -> &'static str {
        "2025-03-26"
    }
    
    fn supports_feature(&self, feature: Feature) -> bool {
        matches!(feature,
            Feature::AsyncTools |
            Feature::BatchRequests |
            Feature::StreamableHttp |
            Feature::ObjectCapabilities |
            Feature::ToolCancellation
        )
    }
    
    fn capability_format(&self) -> CapabilityFormat {
        CapabilityFormat::Object
    }
    
    fn create_adapter(&self) -> Box<dyn ProtocolAdapter> {
        Box::new(Adapter2025_03_26::new())
    }
    
    fn parent_version(&self) -> Option<Box<dyn ProtocolVersion>> {
        None // Base version for Shadowcat
    }
}

// Newer version inherits and overrides
pub struct V2025_06_18;

impl ProtocolVersion for V2025_06_18 {
    fn version_string(&self) -> &'static str {
        "2025-06-18"
    }
    
    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::BatchRequests => false, // Override: removed
            Feature::StructuredOutput | Feature::Elicitation => true, // New
            _ => V2025_03_26.supports_feature(feature), // Inherit
        }
    }
    
    fn capability_format(&self) -> CapabilityFormat {
        CapabilityFormat::Object // Same as parent
    }
    
    fn create_adapter(&self) -> Box<dyn ProtocolAdapter> {
        Box::new(Adapter2025_06_18::new())
    }
    
    fn parent_version(&self) -> Option<Box<dyn ProtocolVersion>> {
        Some(Box::new(V2025_03_26))
    }
}
```

### Test Declaration Pattern

```rust
// Declarative test metadata
#[derive(Clone)]
pub struct TestMetadata {
    pub name: &'static str,
    pub category: TestCategory,
    pub min_version: Option<&'static str>,
    pub max_version: Option<&'static str>,
    pub required_features: Vec<Feature>,
    pub excluded_features: Vec<Feature>,
}

// Macro for easy test registration
macro_rules! compliance_test {
    (
        name: $name:expr,
        category: $category:expr,
        min_version: $min:expr,
        required_features: [$($feature:expr),*],
        test_fn: $test_fn:expr
    ) => {
        TestCase {
            metadata: TestMetadata {
                name: $name,
                category: $category,
                min_version: Some($min),
                max_version: None,
                required_features: vec![$($feature),*],
                excluded_features: vec![],
            },
            test_fn: Box::new($test_fn),
        }
    };
}

// Usage example
pub fn register_tests() -> Vec<TestCase> {
    vec![
        compliance_test!(
            name: "async_tool_execution",
            category: TestCategory::Tools,
            min_version: "2025-03-26",
            required_features: [Feature::AsyncTools],
            test_fn: test_async_tool_execution
        ),
        
        compliance_test!(
            name: "batch_requests",
            category: TestCategory::Protocol,
            min_version: "2025-03-26",
            excluded_features: [Feature::NoBatching],
            test_fn: test_batch_requests
        ),
        
        compliance_test!(
            name: "structured_output",
            category: TestCategory::Tools,
            min_version: "2025-06-18",
            required_features: [Feature::StructuredOutput],
            test_fn: test_structured_output
        ),
    ]
}
```

### Dynamic Test Selection

```rust
pub struct TestRunner {
    registry: VersionRegistry,
    tests: Vec<TestCase>,
}

impl TestRunner {
    pub fn run_for_version(&self, version_str: &str) -> TestReport {
        let version = self.registry.get(version_str)
            .expect("Unknown version");
        
        let applicable_tests: Vec<_> = self.tests
            .iter()
            .filter(|test| self.is_applicable(test, version))
            .collect();
        
        let mut report = TestReport::new(version_str);
        
        for test in applicable_tests {
            let result = self.run_test(test, version).await;
            report.add_result(test.metadata.name, result);
        }
        
        report
    }
    
    fn is_applicable(&self, test: &TestCase, version: &dyn ProtocolVersion) -> bool {
        // Check version range
        if let Some(min) = test.metadata.min_version {
            if !self.version_gte(version.version_string(), min) {
                return false;
            }
        }
        
        if let Some(max) = test.metadata.max_version {
            if !self.version_lte(version.version_string(), max) {
                return false;
            }
        }
        
        // Check required features
        for feature in &test.metadata.required_features {
            if !version.supports_feature(*feature) {
                return false;
            }
        }
        
        // Check excluded features
        for feature in &test.metadata.excluded_features {
            if version.supports_feature(*feature) {
                return false;
            }
        }
        
        true
    }
}
```

## Adding a New Version

When MCP releases a new version (e.g., 2025-09-XX), the process is:

### 1. Create Version Module

```rust
// src/compliance/versions/v2025_09_xx.rs
pub struct V2025_09_XX;

impl ProtocolVersion for V2025_09_XX {
    fn version_string(&self) -> &'static str {
        "2025-09-XX"
    }
    
    fn supports_feature(&self, feature: Feature) -> bool {
        match feature {
            Feature::NewFeatureX => true, // New in this version
            _ => V2025_06_18.supports_feature(feature), // Inherit
        }
    }
    
    // ... rest of implementation
}
```

### 2. Define Feature Differences

```rust
// Add new features to enum
pub enum Feature {
    // ... existing features ...
    NewFeatureX,
    EnhancedValidationY,
}

// Update feature detection
impl V2025_09_XX {
    fn get_changes_from_parent(&self) -> VersionChanges {
        VersionChanges {
            added_features: vec![Feature::NewFeatureX],
            removed_features: vec![],
            modified_behaviors: vec![
                ("tools/call", "Added new response field 'metrics'"),
            ],
        }
    }
}
```

### 3. Add Version-Specific Tests

```rust
// src/compliance/tests/v2025_09_xx.rs
pub fn register_v2025_09_tests() -> Vec<TestCase> {
    vec![
        compliance_test!(
            name: "new_feature_x",
            category: TestCategory::NewFeatures,
            min_version: "2025-09-XX",
            required_features: [Feature::NewFeatureX],
            test_fn: test_new_feature_x
        ),
    ]
}
```

### 4. Register in Main

```rust
// src/compliance/mod.rs
pub fn initialize_compliance() -> ComplianceFramework {
    let mut registry = VersionRegistry::new();
    
    // Add new version
    registry.register(Box::new(V2025_09_XX));
    
    // Tests auto-discover applicable versions
    let tests = collect_all_tests();
    
    ComplianceFramework { registry, tests }
}
```

## Configuration-Driven Approach

### Version Configuration File

```toml
# versions.toml
[versions."2025-03-26"]
parent = null
features = [
    "async_tools",
    "batch_requests",
    "streamable_http",
    "object_capabilities",
]

[versions."2025-06-18"]
parent = "2025-03-26"
added_features = ["structured_output", "elicitation"]
removed_features = ["batch_requests"]

[versions."2025-09-XX"]
parent = "2025-06-18"
added_features = ["new_feature_x"]
modified_behaviors = [
    { method = "tools/call", change = "Added metrics field" }
]
```

### Auto-Generation from Config

```rust
// Build script to generate version modules from config
// build.rs
fn main() {
    let config = read_versions_config("versions.toml");
    
    for (version, spec) in config.versions {
        generate_version_module(&version, &spec);
    }
}
```

## Benefits of This Architecture

1. **Zero-Code Version Addition** - New versions can be added via configuration
2. **Automatic Test Discovery** - Tests know which versions they apply to
3. **Single Source of Truth** - Version features defined once
4. **Inheritance Model** - Reduces duplication between versions
5. **Type Safety** - Compile-time verification of version features
6. **Clear Migration Path** - Differences between versions are explicit

## Example: Running Compliance Tests

```rust
#[tokio::test]
async fn test_compliance_all_versions() {
    let framework = initialize_compliance();
    
    // Test all supported versions
    for version in ["2025-03-26", "2025-06-18"] {
        let report = framework.run_for_version(version).await;
        
        println!("Version {} Compliance:", version);
        println!("  Passed: {}/{}", report.passed, report.total);
        println!("  Coverage: {:.1}%", report.coverage_percentage());
        
        assert!(report.is_compliant(), "Version {} not compliant", version);
    }
}

// Or test specific version
#[tokio::test]
async fn test_latest_version_compliance() {
    let framework = initialize_compliance();
    let report = framework.run_for_version("2025-06-18").await;
    
    // Generate detailed report
    report.write_json("compliance-2025-06-18.json")?;
    report.write_markdown("compliance-2025-06-18.md")?;
    
    assert!(report.is_compliant());
}
```

## Summary

This version-agnostic architecture ensures:
- **Easy version addition** - New MCP versions require minimal code
- **Automatic adaptation** - Tests self-configure based on version features
- **Clear documentation** - Version differences are explicit and traceable
- **Future-proof** - Ready for quarterly MCP updates
- **Maintainable** - Centralized version management reduces complexity

---

*Created: 2025-08-23*
*Purpose: Support frequent MCP version updates*
*Target: 2025-03-26 and 2025-06-18, with easy addition of future versions*