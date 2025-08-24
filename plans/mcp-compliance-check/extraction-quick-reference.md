# MCP Extraction Quick Reference Card

## 🎯 Current Location Check
```bash
pwd  # Must be: /Users/kevin/src/tapwire/shadowcat-mcp-compliance
git branch --show-current  # Must be: feat/mcpspec
```

## 📝 Extraction Order (B.0 Task)
1. ✅ constants.rs (no deps)
2. ✅ version.rs (needs constants)
3. ✅ types.rs (needs serde)
4. ✅ messages.rs (needs types)
5. ⏸️ validation.rs (if time)
6. ⏸️ builder.rs (if time)

## 🔧 Quick Fixes for Common Issues

### Import Errors
```rust
// Replace these:
use crate::error::Result;     → pub type Result<T> = std::result::Result<T, Error>;
use crate::something::Thing;  → // Remove or find Thing in extracted files
pub(crate)                    → pub
```

### Missing Error Type
```rust
// Add to lib.rs:
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
```

### Feature Flags
```rust
#[cfg(feature = "shadowcat-xyz")]  → // Remove entire block
```

## ✅ After Each File
```bash
cargo check              # Compiles?
cargo test smoke_test    # Basic test passes?
git add -A && git commit -m "wip: extracted X"  # Save progress
```

## 🎯 Success Indicators
- ✅ `cargo check` - No errors
- ✅ `cargo test` - Tests pass
- ✅ No "shadowcat" in src/
- ✅ No "use crate::" in src/
- ✅ Can create types in example

## 📊 Progress Tracker
```
[x] Setup crate structure
[x] constants.rs extracted
[x] version.rs extracted  
[x] types.rs extracted
[ ] messages.rs extracted
[ ] All tests passing
[ ] Fixtures parsing
```

## 🚀 Final Validation
```bash
# In crates/mcp directory:
cargo test --all
cargo doc --open
cargo run --example demo
```

## 💾 Commit Message
```bash
git add -A
git commit -m "feat(mcp): extract core types from shadowcat

- Extract constants.rs, version.rs, types.rs, messages.rs
- Remove shadowcat-specific dependencies
- Add basic integration tests
- Fixtures parse successfully
- Milestone 1 complete"

git push origin feat/mcpspec
```

---
Keep this card visible during extraction!