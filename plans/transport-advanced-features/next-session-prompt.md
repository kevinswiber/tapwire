# Next Session: ProcessManager Implementation

## Context
Phase 1 analysis and design are complete. We have a clear understanding of the current subprocess handling limitations and a detailed design for ProcessManager integration. The implementation will provide graceful shutdown, health monitoring, and improved lifecycle management.

## Session Objectives
Implement the core ProcessManager integration as designed, focusing on backward compatibility and graceful shutdown capabilities.

## Tasks for This Session

### Task P.3: Implement ProcessManager in SubprocessOutgoing (2h)
Integrate ProcessManager with the transport layer following the design in `analysis/process-manager-design.md`.

**Sub-tasks:**
1. Extend error types for process management
2. Modify ProcessManager for I/O handle access
3. Update SubprocessOutgoing with optional ProcessManager
4. Update StdioRawOutgoing with graceful shutdown
5. Write comprehensive tests

## Key Implementation Points

### Backward Compatibility
- ProcessManager must be optional
- Existing code must work without changes
- Default behavior unchanged when ProcessManager is None

### Graceful Shutdown
- Send SIGTERM before SIGKILL on Unix
- Configurable timeout (default 5 seconds)
- Proper cleanup of I/O tasks

### Error Handling
- New error variants in TransportError
- Clear distinction between transport and process errors
- Proper error propagation

## Deliverables
- [ ] Modified transport implementations with ProcessManager support
- [ ] Enhanced error types
- [ ] Graceful shutdown implementation
- [ ] Unit and integration tests
- [ ] All tests passing
- [ ] No clippy warnings

## Success Criteria
- Existing tests continue to pass
- New tests verify ProcessManager integration
- Graceful shutdown works as designed
- Code is backward compatible
- Performance impact is minimal

## References
- Analysis: `@plans/transport-advanced-features/analysis/subprocess-handling-analysis.md`
- Design: `@plans/transport-advanced-features/analysis/process-manager-design.md`
- Task Details: `@plans/transport-advanced-features/tasks/P.3-implement-process-manager.md`
- Tracker: `@plans/transport-advanced-features/transport-advanced-features-tracker.md`

## Time Estimate
2 hours for implementation and testing

## Next Steps After This Session
If implementation is successful:
- Phase 2: Batch Message Support (Task B.1-B.4)
- Or continue with monitoring capabilities for ProcessManager

## Notes
- This is an enhancement, not a critical fix
- Focus on core integration first, monitoring can be added later
- Ensure thorough testing of backward compatibility
- Document any API changes clearly