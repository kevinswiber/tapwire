# Task C.0: Environment Setup

## Objective
Set up development environment for Wassette-Shadowcat integration, including building Wassette and creating test WebAssembly components.

## Process

### 1. Build Wassette
```bash
# Clone Wassette if not already available
git clone https://github.com/microsoft/wassette.git ../wassette-test
cd ../wassette-test
cargo build --release

# Add to PATH for testing
export PATH=$PATH:$(pwd)/target/release
```

### 2. Create Test WebAssembly Components
- Use existing Wassette examples as starting point
- Focus on simple tool functions for testing
- Ensure components use stdio transport

### 3. Set Up Shadowcat Integration Branch
- Create feature branch in shadowcat-wassette worktree
- Prepare directory structure for new modules
- Set up test infrastructure

## Deliverables
- [ ] Wassette binary available and functional
- [ ] Test WebAssembly components ready
- [ ] Shadowcat branch ready for implementation
- [ ] Integration test framework prepared

## Success Criteria
- Can run `wassette serve --stdio` successfully
- Can load and execute test WebAssembly components
- Shadowcat development environment configured
- Basic integration test harness in place