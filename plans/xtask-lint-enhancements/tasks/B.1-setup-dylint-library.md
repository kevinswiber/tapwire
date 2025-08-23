# Task B.1: Setup Dylint Library

## Objective
Create the basic dylint library structure for shadowcat custom lints.

## Key Questions
- What's the minimal dylint setup required?
- How do we configure workspace integration?
- What's the build/test workflow?

## Process

### 1. Install dylint tools
```bash
cargo install cargo-dylint dylint-link
```

### 2. Create library structure
```bash
cd shadowcat
cargo dylint new shadowcat_lints
cd shadowcat_lints
```

### 3. Configure workspace integration
Add to shadowcat's Cargo.toml:
```toml
[workspace.metadata.dylint]
libraries = [
    { path = "shadowcat_lints" }
]
```

### 4. Set up basic lint structure
- Update Cargo.toml with dependencies
- Create initial lib.rs with lint registration
- Add utilities module for common helpers

### 5. Verify compilation
```bash
cargo build --lib
cargo dylint list
```

## Deliverables
- [ ] shadowcat_lints/ directory with Cargo.toml
- [ ] Basic lib.rs with dylint setup
- [ ] Workspace metadata configuration
- [ ] Successful `cargo dylint list` output

## Success Criteria
- [ ] Library compiles without errors
- [ ] `cargo dylint list` shows the library
- [ ] Can run `cargo dylint --lib shadowcat_lints`

## Notes
- Reference: https://github.com/trailofbits/dylint/tree/master/examples
- Use rustc_lint and rustc_hir for lint infrastructure
- Start with minimal dependencies, add as needed