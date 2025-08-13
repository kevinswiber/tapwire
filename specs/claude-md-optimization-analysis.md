# CLAUDE.md Optimization Analysis

## Analysis of Current Files

### shadowcat/CLAUDE.md (358 lines)
**Strengths:**
- ✅ Excellent clippy compliance section with concrete examples
- ✅ Clear, actionable commands organized by purpose
- ✅ Specific error patterns to avoid
- ✅ Pre-commit checklist is concrete and ordered
- ✅ Good use of code examples with ❌ BAD / ✅ GOOD pattern

**Areas for Optimization:**
- Could benefit from more bullet points vs. prose in some sections
- Architecture section could be more concise
- Some redundancy in command examples

### Main CLAUDE.md (379 lines)
**Strengths:**
- ✅ Clear git submodule workflow instructions
- ✅ Well-structured planning process documentation
- ✅ Good security requirements section
- ✅ Performance targets are specific and measurable

**Areas for Optimization:**
- Very long for a root CLAUDE.md
- Planning process (lines 304-368) might be better as a separate import
- Some duplication with shadowcat/CLAUDE.md
- Architecture overview could be more concise

## Optimization Recommendations

### 1. Split into Modular Files
Create specialized memory files and use imports:

```markdown
# Main CLAUDE.md (reduced to ~150 lines)
@.claude/planning-process.md
@.claude/security-requirements.md
@shadowcat/CLAUDE.md
```

### 2. Consolidate Duplicate Content
Both files have overlapping content that should be unified:
- Development commands
- Architecture overview
- Testing patterns
- Error handling

### 3. Optimize shadowcat/CLAUDE.md

#### Before (Current):
```markdown
### Code Quality

```bash
cargo fmt              # Format code
cargo fmt -- --check   # Check formatting in CI
cargo clippy --all-targets -- -D warnings
cargo doc --open       # Generate and view docs
```
```

#### After (Optimized):
```markdown
### Code Quality
- Format: `cargo fmt`
- Check format: `cargo fmt -- --check`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Docs: `cargo doc --open`
```

### 4. Create Supporting Memory Files

#### .claude/planning-process.md
Move lines 304-368 from main CLAUDE.md here. This is rarely needed in every session.

#### .claude/security-requirements.md
Move security section (lines 263-285) here for better organization.

#### .claude/git-workflow.md
Consolidate all git-related commands and workflows.

### 5. Restructure Main CLAUDE.md

```markdown
# Tapwire Project

## Quick Start
- Clone: `git clone --recursive <repo>`
- Setup: `cd shadowcat && cargo build`
- Test: `cargo test`
- Run: `cargo run -- forward stdio -- echo '{"jsonrpc":"2.0","method":"ping","id":1}'`

## Project Structure
- `tapwire/`: Platform vision and coordination
- `shadowcat/`: Core Rust proxy (git submodule)
- `plans/`: Feature planning documents
- `specs/`: Technical specifications

## Development Workflow
@.claude/git-workflow.md
@shadowcat/CLAUDE.md

## Architecture
- MCP Protocol: v2025-11-05
- Core: Shadowcat proxy (Rust/Tokio)
- Storage: SQLite for sessions/tapes
- Auth: OAuth 2.1 compliant

## Critical Rules
- **NEVER** add Claude as git co-author
- **NEVER** forward client tokens upstream
- **ALWAYS** run clippy before commits
- **ALWAYS** commit to shadowcat submodule first

## Planning Process
@.claude/planning-process.md

## Security
@.claude/security-requirements.md
```

### 6. Optimize shadowcat/CLAUDE.md Structure

```markdown
# Shadowcat - MCP Proxy

## Essential Commands
### Development
- Test: `cargo test`
- Test specific: `cargo test transport::stdio -- --nocapture`
- Watch: `cargo watch -x check -x test -x run`
- Debug: `RUST_LOG=shadowcat=debug cargo run`

### Quality Gates (MUST PASS)
1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`

## Clippy Compliance Guide
[Keep existing excellent clippy section but consolidate examples]

## Architecture
- Transport: stdio, HTTP/SSE via trait
- Proxy: Forward/reverse with auth
- Session: Thread-safe, SQLite backed
- Interceptor: Pause/modify/block chain

## Quick Patterns
### Add Transport
1. Implement trait in `src/transport/new.rs`
2. Export in `mod.rs`
3. Update CLI
4. Add integration tests

### Error Handling
```rust
use crate::error::Result;
use anyhow::Context;
something.await.context("Failed during X")?;
```

## Performance Targets
- Latency: < 5% p95 overhead
- Memory: < 100KB/session
- Startup: < 50ms
```

### 7. Key Improvements Summary

1. **Reduce main CLAUDE.md from 379 to ~150 lines**
2. **Use imports for modular organization**
3. **Convert prose to bullet points**
4. **Remove redundant information**
5. **Keep clippy guide (it's excellent)**
6. **Focus on most-used commands**
7. **Move planning process to separate file**

### 8. Implementation Priority

1. **High Priority**
   - Create .claude/ directory for imports
   - Split planning process into separate file
   - Convert commands to bullet points
   - Remove duplication between files

2. **Medium Priority**
   - Consolidate architecture sections
   - Create git-workflow.md
   - Optimize command grouping

3. **Low Priority**
   - Further modularization
   - Team-wide convention files
   - Enterprise-level standards