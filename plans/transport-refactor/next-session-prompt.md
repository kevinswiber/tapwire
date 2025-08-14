# Next Session: Test Coverage Analysis

## Context
The transport refactor is complete (Session 7), but we deleted several test files during the cleanup. We need to analyze what test coverage was lost and ensure all important scenarios are still tested in the new directional transport architecture.

## Objective
Analyze deleted test files to ensure we haven't lost important test coverage, and create any missing tests for the new IncomingTransport/OutgoingTransport system.

## Files Deleted (from git diff --name-only 298b72a)
```
src/transport/size_limit_tests.rs
src/transport/validation_test.rs
tests/integration_forward_proxy_sse.rs
tests/pause_resume_test.rs
tests/sse_interceptor_test.rs
tests/sse_transport_test.rs
tests/transport_regression_suite.rs
```

## Modified Test Files
```
tests/integration_api_mock.rs
tests/version_negotiation_test.rs
```

## Tasks (5 hours)

### 1. Analyze Deleted Test Coverage (1h)
For each deleted test file:
- Review what scenarios were being tested
- Document the test's purpose and importance
- Note any critical functionality that needs coverage

### 2. Map to New Architecture (2h)
- Identify where each test scenario should live in the new architecture
- Check if equivalent tests already exist for directional transports
- Create a gap analysis document

### 3. Implement Missing Tests (2h)
- Write new tests for any critical gaps identified
- Focus on:
  - Message size limits
  - Transport validation
  - SSE/streaming scenarios (if still relevant)
  - Pause/resume functionality
  - Regression scenarios

## Key Questions to Answer
1. **Size Limits**: Are message size limits still enforced in directional transports?
2. **Validation**: Is transport validation logic covered in the new system?
3. **SSE/Streaming**: Since we removed SSE transport, is streaming still supported via StreamableHttp?
4. **Pause/Resume**: Is pause/resume functionality still needed and tested?
5. **Regression Suite**: What regression scenarios need to be preserved?

## Success Criteria
- [ ] All deleted test scenarios analyzed
- [ ] Gap analysis document created
- [ ] Critical missing tests identified
- [ ] New tests written for gaps
- [ ] All tests passing
- [ ] Documentation updated

## References
- Tracker: `plans/transport-refactor/transport-refactor-tracker.md`
- See Phase 8 in tracker for detailed task breakdown

## Note
The transport refactor is functionally complete with 788 unit tests passing. This session is about ensuring we haven't lost important test coverage during the cleanup.