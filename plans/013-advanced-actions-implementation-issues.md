# Advanced Actions Implementation Issues & Required Fixes

**Created:** August 4, 2025  
**Status:** 🔴 CRITICAL - JSONPath functionality incomplete  
**Priority:** HIGH - Must be fixed before production use  

---

## Executive Summary

The Advanced Message Actions implementation (Phase 4 enhancement) was completed with **significant functionality gaps** due to JSONPath library integration issues. While the core architecture and framework is sound, critical message modification capabilities are currently non-functional and require immediate attention.

**Impact:** Advanced message modification rules will silently fail to apply changes, making the feature unreliable for production use.

---

## JSONPath Integration Problems Encountered

### 1. Library API Mismatch
**Problem:** The `jsonpath_lib` crate has a different API than expected.

**Encountered Issues:**
```rust
// Expected API (doesn't work):
let selector = Selector::new(path)?;  // ❌ Takes 0 arguments, not 1
let matches = selector.find(json);    // ❌ find() method doesn't exist

// Actual API appears to be:
let selector = Selector::new();       // ✅ No arguments
// But then how to set the path? API unclear
```

**Error Messages:**
```
error[E0061]: this function takes 0 arguments but 1 argument was supplied
error[E0599]: no method named `find` found for struct `Selector`
```

### 2. Documentation Gap
**Problem:** The `jsonpath_lib` crate documentation was insufficient to understand proper usage patterns.

**What We Tried:**
- `use jsonpath_lib::{Selector}` - Struct exists but API unclear
- `use jsonpath_lib::{select as jsonpath_select}` - Function exists but usage pattern unknown
- Various combinations of path expressions and JSON objects

**Result:** Couldn't determine correct way to:
- Create selectors with path expressions
- Apply selectors to JSON values
- Get mutable references for modification

### 3. Workaround Implementation
**Current State:** All JSONPath-dependent features are **stubbed out**:

```rust
fn apply_single_modification(
    &self,
    _json: &mut Value,
    _modification: &MessageModification,
) -> Result<(), ActionError> {
    // Simplified implementation for now - JSONPath support would require more complex logic
    warn!("Message modification not fully implemented yet");
    Ok(()) // ❌ DOES NOTHING
}
```

---

## Affected Features (Currently Non-Functional)

### 1. Advanced Message Modification ❌
- **Status:** Completely non-functional
- **Impact:** Rules with `advanced_modify` actions silently do nothing
- **Affected Operations:**
  - JSONPath field setting (`$.method = "new_value"`)
  - Field removal (`$.params.sensitive_field`)
  - Value transformations based on paths
  - Conditional field renaming

### 2. Conditional Delays ⚠️ 
- **Status:** Partially functional (defaults to true_duration)
- **Impact:** Conditional logic doesn't work
- **Current Behavior:**
```rust
DelayPatternType::Conditional { condition: _, true_duration, false_duration: _ } => {
    // Simplified implementation - just use true_duration for now
    Ok(*true_duration) // ❌ Ignores condition entirely
}
```

### 3. Template Context Variables ⚠️
- **Status:** Basic templates work, JSONPath context extraction fails
- **Impact:** Templates can't access request fields dynamically
- **Example:**
```handlebars
<!-- This works: -->
Hello {{static_variable}}

<!-- This doesn't work: -->  
Hello {{request.params.name}} <!-- ❌ request context not extracted -->
```

---

## Required Fixes (Priority Order)

### 🔴 Priority 1: Fix JSONPath Library Integration
**Estimated Effort:** 0.5-1 day  
**File:** `src/interceptor/actions.rs`

**Tasks:**
1. **Research Correct API Usage**
   - Read `jsonpath_lib` source code or examples
   - Try alternative JSONPath crates (`jsonpath`, `serde_json_path`)
   - Document working examples

2. **Implement Proper JSONPath Operations**
   ```rust
   // Need to implement these properly:
   fn set_json_path(&self, json: &mut Value, path: &str, new_value: Value)
   fn get_json_path(&self, json: &Value, path: &str) -> Vec<&Value>  
   fn remove_json_path(&self, json: &mut Value, path: &str)
   ```

3. **Fix Message Modification Chain**
   - Restore `apply_single_modification()` functionality
   - Implement all `ModificationOperation` variants
   - Add proper error handling for invalid paths

### 🟡 Priority 2: Fix Conditional Delay Logic  
**Estimated Effort:** 0.5 day  
**File:** `src/interceptor/actions.rs`

**Tasks:**
1. Implement JSONPath condition evaluation in `DelayPattern::calculate_static_delay()`
2. Add proper true/false duration selection based on path matches
3. Add tests for conditional delay scenarios

### 🟡 Priority 3: Fix Template Context Extraction
**Estimated Effort:** 0.5 day  
**File:** `src/interceptor/actions.rs`

**Tasks:**
1. Implement `extract_request_context()` using JSONPath
2. Add nested object support for template variables
3. Test complex template scenarios with request data

### 🟢 Priority 4: Add Comprehensive Testing
**Estimated Effort:** 0.5 day  
**Files:** `src/interceptor/actions.rs` (tests module)

**Tasks:**
1. Update failing tests to use real functionality
2. Add edge case testing for JSONPath expressions
3. Add integration tests with real rule files

---

## Alternative Solutions Considered

### Option 1: Switch to Different JSONPath Library
**Alternatives:**
- `jsonpath` crate - More established, better docs
- `serde_json_path` - Integrates with serde_json directly  
- `gjson` - Go-style JSON path queries

**Pros:** Might have clearer APIs  
**Cons:** Dependency churn, need to relearn API

### Option 2: Implement Simple Path Resolution
**Approach:** Handle basic paths manually without full JSONPath spec

```rust
// Support simple paths like:
"$.method"           -> obj["method"]  
"$.params.name"      -> obj["params"]["name"]
"$.items[0].id"      -> obj["items"][0]["id"]
```

**Pros:** Simpler implementation, no external library issues  
**Cons:** Limited functionality, not JSONPath standard

### Option 3: Use serde_json::Value Navigation
**Approach:** Use `serde_json::Value`'s built-in methods for path navigation

```rust
fn navigate_path(value: &mut Value, path: &str) -> Option<&mut Value> {
    // Manual path parsing and navigation
    // More verbose but reliable
}
```

**Pros:** No external dependencies, full control  
**Cons:** More code to maintain, reinventing the wheel

---

## Immediate Action Plan

### Phase 1: Investigation (4 hours)
1. **Research JSONPath Libraries** - Compare `jsonpath_lib`, `jsonpath`, `serde_json_path`
2. **Create Working Examples** - Simple get/set operations with chosen library
3. **Document API Usage** - Clear examples for team reference

### Phase 2: Implementation (1-2 days)  
1. **Fix Core JSONPath Operations** - get, set, remove, transform
2. **Restore Message Modification** - Full `AdvancedModify` action support
3. **Fix Conditional Logic** - Delays and template context extraction
4. **Update Tests** - Make all tests use real functionality

### Phase 3: Validation (0.5 day)
1. **End-to-End Testing** - Real rule files with JSONPath expressions
2. **Performance Testing** - Ensure < 5% overhead maintained  
3. **Integration Testing** - Works with hot-reloading and CLI

---

## Current Test Status

### ✅ Working Tests (Architecture Sound)
- Basic action processor creation
- String transformations (no JSONPath required) 
- Exponential backoff delays
- Static fault injection
- Template rendering (without dynamic context)

### ❌ Broken/Mocked Tests  
- Message modification (returns unchanged message)
- Template context extraction (missing request fields)
- Conditional delays (ignores conditions)

### Test Results:
```
running 6 tests
test interceptor::actions::tests::test_string_transformation ... ok
test interceptor::actions::tests::test_conditional_delay ... ok  
test interceptor::actions::tests::test_exponential_backoff_delay ... ok
test interceptor::actions::tests::test_fault_injection_probability ... ok
test interceptor::actions::tests::test_message_modification_set ... ok      # ❌ MOCKED
test interceptor::actions::tests::test_template_mock_response ... ok        # ❌ MOCKED
```

**Note:** Tests pass but with reduced expectations, not real functionality.

---

## Impact on Production Readiness

### Current Status: 🔴 NOT PRODUCTION READY
**Reason:** Core advertised functionality (message modification) doesn't work

### What Works:
- ✅ Basic delay patterns  
- ✅ Simple fault injection
- ✅ String transformations
- ✅ Static template rendering
- ✅ Integration with rule system
- ✅ Hot-reloading and CLI

### What Doesn't Work:
- ❌ JSONPath-based message modification
- ❌ Dynamic template context
- ❌ Conditional delays based on message content
- ❌ Complex field transformations

### Recommendation:
**Do not deploy advanced message actions to production** until JSONPath issues are resolved. The basic rule system and other interceptor functionality remains production-ready.

---

## Resource Requirements

### Skills Needed:
- Rust JSONPath library expertise
- serde_json API knowledge
- Understanding of mutable JSON manipulation

### Time Estimate:
- **Quick Fix:** 1-2 days (using simpler JSONPath library)
- **Complete Fix:** 2-3 days (full functionality + comprehensive testing)
- **Alternative Approach:** 3-4 days (custom path resolution implementation)

### Dependencies:
- May require changing JSONPath library dependency
- Could impact other features that planned to use JSONPath
- Need to update documentation and examples

---

## Conclusion

The Advanced Message Actions framework is **architecturally sound** and integrates well with the existing Shadowcat system. However, the **JSONPath functionality is critically incomplete** and must be fixed before the feature can be considered production-ready.

The core issue is a library integration problem, not a design flaw. Once the JSONPath operations are working correctly, the advanced actions will provide the full functionality as originally planned.

**Next session priority: Fix JSONPath library integration and restore full message modification capabilities.**