# Config Validation Enhancement Plan

## 🎯 Objective

Transform Shadowcat's generic config validation errors into rich, actionable error types with workload-based defaults and user guidance.

## 📋 Quick Links

- [Main Tracker](config-validation-enhancement-tracker.md) - Comprehensive plan and progress tracking
- [Tasks](tasks/) - Individual task descriptions
- [Analysis](analysis/) - Research and findings

## 🚀 Quick Start

If you're picking up this plan:

1. **Review the tracker** - Check current status and completed tasks
2. **Check analysis** - Review any findings from previous sessions
3. **Pick next tasks** - Use the tracker to identify next tasks
4. **Update progress** - Mark tasks as you complete them

## 📊 Current Status

**Phase**: Planning  
**Progress**: 0% (0/15 tasks)  
**Estimated Remaining**: 16-24 hours

### Next Steps
1. Audit current error usage patterns
2. Design specific error variants
3. Document affected files for rename

## 🎨 Key Changes

### Before
```rust
// Generic, unhelpful errors
Error::Invalid("Invalid port in server bind address 'localhost:80': permission denied")

// Redundant naming
use shadowcat::config::ShadowcatConfig;

// No smart defaults - users configure everything
```

### After
```rust
// Rich, actionable errors
Error::InvalidPort {
    port: 80,
    reason: PortError::Privileged,
    suggestion: "Use port above 1024 or run with sudo"
}

// Clean naming
use shadowcat::config::Config;

// Workload-based defaults
let config = Config::for_workload(Workload::HighThroughput);
```

## 📁 Plan Structure

```
config-validation-enhancement/
├── README.md                              # This file
├── config-validation-enhancement-tracker.md  # Main tracking document
├── tasks/                                  # Individual task files
│   ├── A.0-audit-error-usage.md
│   ├── A.1-design-error-variants.md
│   └── ...
└── analysis/                              # Research and findings
    ├── current-error-patterns.md
    ├── affected-files.md
    └── workload-profiles.md
```

## ⚠️ Important Considerations

1. **Breaking Changes**: Renaming `ShadowcatConfig` will affect many files
2. **API Compatibility**: Public API must remain stable
3. **Performance**: Validation should not become slower
4. **Simplicity**: Don't over-engineer - start with common cases

## 🔧 Development Workflow

1. **Error Enhancement**:
   ```bash
   # Find validation patterns
   grep -r "Error::Invalid" src/config --include="*.rs"
   
   # Test changes
   cargo test config::
   ```

2. **Rename Operation**:
   ```bash
   # Find all references
   grep -r "ShadowcatConfig" src/ --include="*.rs"
   
   # Rename and test
   cargo check --all-targets
   ```

3. **Workload Defaults**:
   ```bash
   # Test different profiles
   cargo run -- --config-workload=high-throughput
   ```

## 📈 Success Metrics

- **Error Clarity**: Users understand what went wrong and how to fix it
- **Configuration Speed**: Common setups achievable with one line
- **Test Coverage**: All error variants have tests
- **Documentation**: Every error has help text

## 🚦 Decision Points

### Error Granularity
- **Option A**: Specific variant for every validation failure
- **Option B**: Categories of errors with string details
- **Recommendation**: Start with Option B, refine common cases to Option A

### Workload Profiles
- **Option A**: Hardcoded profiles (dev, prod, high-throughput, low-latency)
- **Option B**: Composable settings (base + modifiers)
- **Recommendation**: Start with Option A, consider Option B later

### Performance Warnings
- **Option A**: Separate warning system from errors
- **Option B**: Error variants with severity levels
- **Recommendation**: Option A for clarity

## 📚 References

- [Original Feedback](feedback/) - Claude's suggestions on config errors
- [Error Boundary Fix](../archive/completed-2025-08-22-error-boundary-fix/) - Related error work
- [Thiserror Docs](https://docs.rs/thiserror/latest/thiserror/) - Error derivation

## 💡 Tips

- Start with the most frequently hit errors
- Keep error messages concise but actionable
- Test with real misconfigurations
- Consider adding a `--validate-config` CLI flag