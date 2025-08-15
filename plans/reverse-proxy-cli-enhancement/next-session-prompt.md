# Next Session: Reverse Proxy CLI Enhancement - Implementation Phase 1

## Context
The design phase for the Shadowcat reverse proxy CLI enhancement has been completed. All design documents and initial task files have been created:
- ✅ CLI Design Proposal (`analysis/cli-design-proposal.md`)
- ✅ Configuration File Format (`analysis/config-file-format.md`)
- ✅ Load Balancing Task (`tasks/B.2-load-balancing.md`)
- ✅ Recording Config Task (`tasks/B.3-recording-config.md`)

## Session Goals
Begin Phase 1 implementation focusing on core features that enable production use:
1. Multiple upstream support with configuration parsing
2. Load balancing strategy implementation
3. Basic configuration file support

## Tasks for This Session

### Primary Task: Multiple Upstream Support (4 hours)
Implement task B.1 - Multiple Upstreams:
- Modify CLI to accept multiple `--upstream` flags
- Parse upstream specifications (format: `name=url,weight=N`)
- Update configuration building logic
- Ensure backward compatibility with single upstream
- Add validation for upstream configurations
- Write comprehensive tests

### Secondary Task: Load Balancing Implementation (3 hours)
If time permits, begin task B.2 - Load Balancing:
- Expose `ReverseLoadBalancingStrategy` enum via CLI
- Implement `--load-balancing` flag with strategy selection
- Add weight parsing for weighted strategies
- Implement connection tracking for least-connections
- Add basic sticky session support

## Key References
- **Tracker**: `plans/reverse-proxy-cli-enhancement/reverse-proxy-cli-enhancement-tracker.md`
- **Current State**: `plans/reverse-proxy-cli-enhancement/analysis/current-state.md`
- **Reverse Proxy Module**: `shadowcat/src/proxy/reverse.rs`
- **CLI Module**: `shadowcat/src/cli/reverse.rs`

## Important Constraints
1. **Backward Compatibility**: All existing CLI flags must continue to work exactly as they do now
2. **Progressive Disclosure**: Basic usage should remain simple; advanced features should be optional
3. **Configuration Priority**: CLI args > Config file > Environment vars > Defaults
4. **Error Messages**: All validation errors must be clear and actionable

## Design Principles to Follow
1. **Consistency**: Match patterns from other Shadowcat commands
2. **Discoverability**: Features should be easy to find in help
3. **Composability**: Options should work well together
4. **Testability**: Design with testing in mind

## Key Files to Modify
- `shadowcat/src/cli/reverse.rs` - CLI argument parsing
- `shadowcat/src/proxy/reverse.rs` - Reverse proxy configuration
- `shadowcat/src/cli/mod.rs` - CLI module exports
- `shadowcat/Cargo.toml` - Dependencies if needed

## Testing Requirements
- Unit tests for upstream parsing logic
- Integration tests for multiple upstreams
- Backward compatibility tests
- Load balancing distribution tests
- Run full test suite: `cargo test`
- Check clippy: `cargo clippy --all-targets -- -D warnings`

## Deliverables Checklist
- [ ] Multiple upstream support implementation
- [ ] Load balancing strategy selection
- [ ] Comprehensive test coverage
- [ ] Updated CLI help text
- [ ] Backward compatibility verified
- [ ] All tests passing
- [ ] No clippy warnings

## Success Criteria
- Design enables all module capabilities to be accessed
- Maintains full backward compatibility
- Provides clear migration path for existing users
- Supports both simple and complex use cases
- Includes comprehensive examples

## Notes from Previous Session
- Current CLI only exposes ~20% of reverse proxy capabilities
- Priority features: multiple upstreams, load balancing, authentication, circuit breakers
- Consider using structured config files for complex setups
- Module already has all features implemented, just need CLI exposure

---

**Estimated Duration**: 5 hours  
**Next Phase**: Implementation of core features (Phase 1)