# Task A.0: Extract mcp-validator Test Cases

## Objective

Extract and document all test cases from mcp-validator to create a comprehensive catalog of MCP compliance scenarios that we can reimplement in Rust.

## Background

The mcp-validator contains 36 test cases across multiple categories. While the implementation has critical bugs, the test scenarios themselves are valuable and represent important compliance points. We need to:
- Understand what each test validates
- Identify protocol version differences
- Categorize tests by functionality
- Note any proxy-specific gaps

## Key Questions to Answer

1. What are all the test scenarios and what do they validate?
2. Which tests are version-specific vs universal?
3. What proxy-specific scenarios are missing?
4. How can we improve upon these tests?

## Step-by-Step Process

### 1. Analysis Phase (30 min)

Explore the mcp-validator test structure:

```bash
# Navigate to validator
cd /Users/kevin/src/tapwire/tools/mcp-validator

# Examine test structure
find mcp_testing/tests -name "*.py" -type f | head -20

# Count test functions
grep -r "^async def test_" mcp_testing/tests/ | wc -l
grep -r "^def test_" mcp_testing/tests/ | wc -l

# Look at test categories
ls -la mcp_testing/tests/
ls -la mcp_testing/tests/base_protocol/
ls -la mcp_testing/tests/features/
```

### 2. Extraction Phase (2 hours)

#### 2.1 Base Protocol Tests

Read and document from `mcp_testing/tests/base_protocol/test_initialization.py`:
- Test name
- Purpose
- Key validations
- Protocol versions affected

#### 2.2 Feature Tests

Extract from feature test files:
- `features/test_tools.py`
- `features/test_async_tools.py`
- `features/dynamic_tool_tester.py`
- `features/dynamic_async_tools.py`

#### 2.3 Specification Coverage

Document from `specification_coverage.py`:
- Compliance requirements
- Protocol-specific validations
- Error scenarios

### 3. Documentation Phase (1 hour)

Create comprehensive test catalog in `analysis/validator-test-catalog.md`:

```markdown
# mcp-validator Test Catalog

## Test Categories

### Base Protocol (X tests)
1. **test_initialization**
   - Purpose: Validates basic initialization flow
   - Validates: Protocol version, capabilities, session creation
   - Versions: All
   - Key assertions: ...

### Tools (X tests)
...

### Async Operations (X tests)
...
```

### 4. Gap Analysis Phase (30 min)

Identify missing test scenarios for proxy:
- Session ID mapping tests
- Multi-upstream failover tests
- Connection pooling tests
- OAuth forwarding tests
- SSE reconnection tests

## Expected Deliverables

### New Files
- `analysis/validator-test-catalog.md` - Complete test case documentation
- `analysis/test-gap-analysis.md` - Missing proxy-specific tests

### Documentation Structure

```markdown
# Test Catalog Structure

## Category: {Category Name}

### Test: {test_name}
**Purpose**: What this test validates
**Protocol Versions**: Which versions this applies to
**Key Validations**:
- Validation point 1
- Validation point 2

**Implementation Notes**:
- Special considerations for Rust
- Improvements to make

**Example Request/Response**:
```json
{example}
```
```

## Success Criteria Checklist

- [x] All ~~36~~ 54 test cases documented (found 54 total tests)
- [x] Test purposes clearly explained
- [x] Protocol version applicability noted
- [x] Key validations identified
- [x] Proxy-specific gaps documented (28 additional tests identified)
- [x] Categorization complete (8 categories)
- [x] Implementation notes added
- [x] Tracker updated with findings

## Risk Assessment

| Risk | Impact | Mitigation | 
|------|--------|------------|
| Missing test context | MEDIUM | Read implementation code for clarity |
| Ambiguous test purpose | LOW | Infer from assertions and test name |

## Duration Estimate

**Total: 4 hours**
- Analysis: 30 minutes
- Extraction: 2 hours
- Documentation: 1 hour
- Gap analysis: 30 minutes

## Dependencies

None - This is the first analysis task

## Integration Points

- **Test Runner**: These tests will be implemented by the runner (Task B.1)
- **Protocol Adapters**: Version-specific tests inform adapter design (Task B.2)
- **Report Generator**: Test categories structure report output (Task B.3)

## Commands Reference

```bash
# Working directory
cd /Users/kevin/src/tapwire/tools/mcp-validator

# Find all test files
find mcp_testing/tests -name "test_*.py" -o -name "*_test*.py"

# Extract test function names
grep -h "^async def test_\|^def test_" mcp_testing/tests/**/*.py | sed 's/.*def //' | sed 's/(.*//' | sort

# View specific test file
cat mcp_testing/tests/base_protocol/test_initialization.py

# Count tests per category
for dir in mcp_testing/tests/*/; do
  echo "$dir:"
  grep -r "def test_" "$dir" | wc -l
done
```

## Example Test Documentation

```markdown
### Test: test_initialization

**Purpose**: Validates that a client can successfully initialize a connection with an MCP server

**Protocol Versions**: All (2024-11-05, 2025-03-26, 2025-06-18)

**Key Validations**:
- Server responds with valid initialization response
- Protocol version is negotiated correctly
- Server capabilities are returned
- Session ID is established (for HTTP transport)

**Test Flow**:
1. Send initialize request with client capabilities
2. Verify response contains required fields
3. Check protocol version matches or is negotiated
4. Validate server capabilities structure

**Rust Implementation Notes**:
- Use `serde_json::Value` for dynamic validation
- Test both forward and reverse proxy modes
- Add timeout handling (not in original)
- Verify session manager integration

**Example Request**:
```json
{
  "jsonrpc": "2.0",
  "id": "init",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-03-26",
    "capabilities": {},
    "clientInfo": {
      "name": "Test Client",
      "version": "1.0.0"
    }
  }
}
```
```

## Follow-up Tasks

After completing this task:
- Task A.1: Map tests to MCP specification requirements
- Task A.2: Design Rust module structure for these tests
- Task A.3: Create additional proxy-specific test scenarios

---

**Task Status**: ✅ Completed
**Created**: 2025-08-23
**Last Modified**: 2025-08-23
**Completed**: 2025-08-23
**Author**: Development Team