# First Extraction Kit - Everything for B.0

## Pre-Flight Checklist
Before starting extraction, verify these are ready:

### 1. Worktree Setup Verification
```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
git status  # Must show: On branch feat/mcpspec
pwd  # Must show: /Users/kevin/src/tapwire/shadowcat-mcp-compliance
```

### 2. Files to Extract (Priority Order)
Start with truly independent files - zero dependencies:

**Wave 1: Constants & Enums (No dependencies)**
- `constants.rs` - Just constants, will compile immediately
- `version.rs` - Version enum and helpers

**Wave 2: Core Types (Minimal dependencies)**
- `types.rs` - JsonRpcId, SessionId, basic types
- `messages.rs` - MessageEnvelope, ProtocolMessage

**Wave 3: If Time (Depends on Wave 2)**
- `validation.rs` - Message validation
- `builder.rs` - Builder patterns

### 3. Minimal Test File
Create this IMMEDIATELY after crate setup:

```rust
// crates/mcp/tests/smoke_test.rs
#[test]
fn it_compiles() {
    // This passing means extraction worked
    assert_eq!(2 + 2, 4);
}
```

### 4. First Real Test (After types.rs)
```rust
// crates/mcp/tests/types_test.rs
use mcp::types::*;

#[test]
fn can_create_json_rpc_id() {
    let num_id = JsonRpcId::Number(42);
    let str_id = JsonRpcId::String("test".to_string());
    
    // If this compiles and runs, extraction worked!
    assert!(matches!(num_id, JsonRpcId::Number(42)));
    assert!(matches!(str_id, JsonRpcId::String(_)));
}
```

### 5. Fixture Validation Test
```rust
// crates/mcp/tests/fixture_test.rs
#[test]
fn fixtures_are_valid_json() {
    let fixtures = [
        include_str!("../../../plans/mcp-compliance-check/fixtures/initialize_request.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/initialize_response.json"),
        include_str!("../../../plans/mcp-compliance-check/fixtures/error_response.json"),
    ];
    
    for fixture in fixtures {
        let _: serde_json::Value = serde_json::from_str(fixture)
            .expect("Fixture should be valid JSON");
    }
}
```

## The Extraction Process (Mechanical Steps)

### Step 1: Create Structure (5 min)
```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
mkdir -p crates/mcp/src crates/mcp/tests
cd crates/mcp
```

### Step 2: Create Cargo.toml (5 min)
```toml
[package]
name = "mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

[dev-dependencies]
tokio = { version = "1.35", features = ["full"] }
```

### Step 3: Create lib.rs (5 min)
```rust
// crates/mcp/src/lib.rs
//! MCP (Model Context Protocol) implementation extracted from shadowcat

pub mod constants;
pub mod types;
// Add more as extracted

pub use constants::*;
pub use types::*;
```

### Step 4: Copy First File - constants.rs (10 min)
```bash
cp src/mcp/constants.rs crates/mcp/src/
```

Then clean it:
- Remove `use crate::` imports
- Remove `pub(crate)`, make `pub`
- Remove shadowcat features

### Step 5: Immediate Validation (2 min)
```bash
cd crates/mcp
cargo check  # Must compile!
cargo test  # Smoke test must pass!
```

### Step 6: Add to Workspace (5 min)
Edit root Cargo.toml:
```toml
[workspace]
members = [".", "crates/mcp"]
```

Test again:
```bash
cargo check --workspace
```

## Success Criteria for First Session

### Milestone 1A: Structure (15 min)
- [ ] Crate created in worktree
- [ ] Cargo.toml configured
- [ ] lib.rs exists
- [ ] Smoke test passes

### Milestone 1B: Constants (15 min)
- [ ] constants.rs extracted
- [ ] Compiles standalone
- [ ] No shadowcat imports

### Milestone 1C: Version (15 min)
- [ ] version.rs extracted
- [ ] Version enum works
- [ ] Helper functions work

### Milestone 1D: Types (30 min)
- [ ] types.rs extracted
- [ ] JsonRpcId works
- [ ] SessionId works
- [ ] Can create all types

### Milestone 1E: Messages (30 min)
- [ ] messages.rs extracted
- [ ] MessageEnvelope works
- [ ] Can parse fixtures

### Success Validation (15 min)
- [ ] All modules compile
- [ ] Basic tests pass
- [ ] Fixtures parse
- [ ] No shadowcat dependencies

## What Could Go Wrong (And Fixes)

### Issue: Circular Dependencies
**Symptom**: `cannot find type X in this scope`
**Fix**: Extract types.rs before messages.rs

### Issue: Missing Error Type
**Symptom**: `use crate::error::Result`
**Fix**: Create simple Result type:
```rust
pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>;
```

### Issue: Feature Flags
**Symptom**: `#[cfg(feature = "shadowcat-something")]`
**Fix**: Remove the feature-gated code for now

### Issue: Internal Macros
**Symptom**: `use crate::impl_some_macro!`
**Fix**: Expand macro manually or skip for now

## Performance Baseline (Before Starting)

Quick measurement of current shadowcat:
```bash
cd /Users/kevin/src/tapwire/shadowcat
time cargo build --release
# Record: X seconds

cargo bench --bench mcp_parsing 2>/dev/null || echo "No benchmarks yet"
# Record any results
```

## The First Hour Plan

**0:00-0:15**: Setup
- Navigate to worktree
- Create crate structure
- Add Cargo.toml
- Create lib.rs
- Add smoke test

**0:15-0:30**: Extract Constants
- Copy constants.rs
- Clean imports
- Verify compilation
- Add to lib.rs

**0:30-0:45**: Extract Version  
- Copy version.rs
- Clean imports
- Verify compilation
- Test version helpers

**0:45-1:15**: Extract Types
- Copy types.rs
- Clean imports
- Fix Result type
- Create type tests
- Verify all types work

**1:15-1:45**: Extract Messages
- Copy messages.rs
- Clean imports
- Fix dependencies
- Test with fixtures

**1:45-2:00**: Validation & Commit
- Run all tests
- Check no shadowcat deps
- Create summary
- Commit to feat/mcpspec

## Definition of "Done" for B.0

```bash
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance/crates/mcp

# These all work:
cargo check ✓
cargo test ✓
cargo doc ✓

# This shows clean extraction:
! grep -r "use crate::" src/  # No crate:: imports
! grep -r "shadowcat" src/     # No shadowcat references

# This proves it works:
echo 'use mcp::types::JsonRpcId;
fn main() { 
    let id = JsonRpcId::Number(1);
    println!("Created: {:?}", id);
}' > examples/demo.rs
cargo run --example demo  # Prints: Created: Number(1)
```

---

*Created: 2025-08-24*
*Purpose: Everything needed for successful first extraction*
*Key: Mechanical process, immediate validation, no surprises*