# Session Success Kit

## Quick Context Loader
When starting a session, load these in order:
1. This file (session-success-kit.md)
2. Current task from tracker
3. Relevant inventory (mcp-extraction or transport-session)
4. Only load other docs if needed

## Key Decisions Already Made
- **Single MCP crate** (not mcp-core, mcp-client, mcp-server)
- **Copy-first approach** (don't modify shadowcat yet)
- **Hybrid architecture** (Client<T>, Server<H>)
- **Type-conscious naming** (stdio::Transport not StdioTransport)
- **Hyper not reqwest** (for SSE support)
- **Per-request streaming** (HandlerResult enum)

## Common Commands Cheatsheet

### 🚨 Git Worktree Commands
```bash
# Navigate to MCP compliance worktree
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
git status  # Should show: On branch feat/mcpspec

# See all worktrees
git worktree list

# Work happens in worktree, not main shadowcat!
pwd  # Should be: /Users/kevin/src/tapwire/shadowcat-mcp-compliance
```

### Extraction Commands
```bash
# Create MCP crate IN WORKTREE
cd /Users/kevin/src/tapwire/shadowcat-mcp-compliance
cargo new --lib crates/mcp
cd crates/mcp

# Add to workspace (edit Cargo.toml in worktree root)
members = [".", "crates/mcp"]

# Test extraction
cargo check --package mcp
cargo test --package mcp

# See what shadowcat uses (from worktree)
rg "use crate::mcp::" src/
rg "pub struct" src/mcp/types.rs

# Quick compile test
echo "fn main() {}" > examples/test.rs
cargo run --example test
```

## File Extraction Checklist
For each file being extracted:
- [ ] Copy to crates/mcp/src/
- [ ] Remove `use crate::` imports
- [ ] Change `pub(crate)` to `pub`
- [ ] Remove shadowcat-specific features
- [ ] Add module to lib.rs
- [ ] Run `cargo check`
- [ ] Add basic test
- [ ] Document public APIs

## Common Extraction Issues & Fixes

### Issue: "cannot find crate"
```rust
// Before (in shadowcat):
use crate::error::Result;

// After (in MCP crate):
pub type Result<T> = std::result::Result<T, Error>;
```

### Issue: "private type in public interface"
```rust
// Change:
pub(crate) struct Foo {}
// To:
pub struct Foo {}
```

### Issue: Feature flags
```rust
// Remove shadowcat-specific features:
#[cfg(feature = "shadowcat-telemetry")]
// Keep protocol features:
#[cfg(feature = "v2025-06-18")]
```

## Testing Patterns
```rust
// Minimal test to verify extraction
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn can_create_type() {
        let _id = JsonRpcId::Number(1);
        let _msg = MessageEnvelope::default();
    }
}
```

## What NOT to Extract
- Anything in `shadowcat/src/proxy/`
- Session management (src/session/)
- Interceptor implementations (keep trait only)
- Transport implementations (initially)
- Recording/replay features
- Telemetry/metrics

## Quick Validation
After each extraction session:
```bash
# In crates/mcp/
cargo check
cargo test
cargo doc --open  # Review API

# Create simple example
cat > examples/demo.rs << 'EOF'
use mcp::types::JsonRpcId;
fn main() {
    let id = JsonRpcId::Number(42);
    println!("Created ID: {:?}", id);
}
EOF
cargo run --example demo
```

## Session Time Management
- **First 15 min**: Load context, understand task
- **Core 90 min**: Execute extraction
- **Last 30 min**: Test, document, commit
- **Final 15 min**: Update tracker, write handoff

## Commit Message Templates
```bash
# For extraction (in worktree)
git add -A
git commit -m "feat(mcp): extract core types from shadowcat

- Copy types.rs with JsonRpcId, SessionId, etc.
- Remove shadowcat-specific dependencies
- Add basic tests for type creation
- Module compiles standalone"

# Push to feature branch
git push origin feat/mcpspec

# For refactoring
git commit -m "refactor(mcp): simplify message builder API

- Remove unnecessary generic parameters
- Add builder methods for common cases
- Improve error messages"
```

## Red Flags to Avoid
1. **Don't extract everything at once** - Just the current task
2. **Don't refactor shadowcat** - Copy and leave it alone
3. **Don't add features** - Just extract what exists
4. **Don't skip tests** - Even minimal tests help
5. **Don't forget workspace** - Add crate to workspace

## Green Flags (You're on Track)
1. ✅ MCP crate compiles standalone
2. ✅ No shadowcat imports in MCP crate
3. ✅ Can create types/call methods
4. ✅ Tests pass (even if minimal)
5. ✅ Clear module structure emerging

## Questions? Check These First
1. **Architecture**: `analysis/architectural-decisions.md`
2. **What to extract**: `analysis/shadowcat-mcp-extraction-inventory.md`
3. **Task details**: `mcp-compliance-check-tracker.md`
4. **MCP spec**: `~/src/modelcontextprotocol/modelcontextprotocol/docs/specification/`

---

*Purpose: Quick reference for successful extraction sessions*
*Keep this open in a tab during work*