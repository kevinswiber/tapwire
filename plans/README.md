# Shadowcat Development Plans

## Active Development

### ⚡ Parallel Work in Progress (2025-08-12)

We are currently running **two development tracks in parallel**:

#### 🚀 Track 1: SSE/MCP Phase 3 - Parser & Correlation
**[Unified Proxy-SSE-Message Tracker](proxy-sse-message-tracker.md)**  
*Session Prompt: `NEXT_SESSION_PROMPT.md`*

Status: **Phase 3 In Progress** | Phase 3: **31% Complete** (M.1-M.2 done, M.3-M.5 pending)  
Current Tasks (using P.x naming in prompt):
- P.2: Schema Validation (4h) - Different from M.3
- P.3: Correlation Store (5h) - Same as M.4
- P.4: Request/Response Matching (4h) - Part of M.4
- P.5: Integrate with Proxy (5h) - Same as M.5

Note: NEXT_SESSION_PROMPT.md uses P.x task numbering

#### 📦 Track 2: CLI Optimization Phase C
**[CLI Optimization Tracker](cli-refactor-optimization/cli-optimization-tracker.md)**  
*Session Prompt: `NEXT_SESSION_PROMPT_CLI_OPTIMIZATION.md`*

Status: **Phase C In Progress** | Overall: **52% Complete** (43 of 73 hours)  
Current Tasks:
- C.2: Configuration File Support (3h)
- C.3: Improve Error Messages (2h)
- C.4: Add Telemetry/Metrics (4h)

These tracks are **independent and can run in parallel** with no risk of conflicts.

### ✅ Recently Completed

#### Transport Context Refactor - COMPLETE!
**[Transport Context Refactor](transport-context-refactor/transport-context-tracker.md)**  
*Successfully completed in 17.5 hours (71% reduction from 60 hour estimate)*

The refactor successfully separated protocol concerns from transport metadata, introducing the MessageEnvelope system that properly handles transport context.

## Plan Structure

```
plans/
├── README.md (this file)
├── proxy-sse-message-tracker.md   # 🚀 READY - Main SSE integration tracker
├── integration-coordination.md     # How SSE and MCP work together
│
├── transport-context-refactor/     # ✅ COMPLETE - Prerequisite finished!
│   ├── transport-context-tracker.md # Refactor tracker (complete)
│   ├── PROGRESS.md                # Detailed completion notes
│   └── analysis/                  # Design documents created
│       ├── migration-strategy-simplified.md
│       └── message-envelope-design.md
│
├── sse-proxy-integration/          # 🚀 READY - SSE Transport Implementation
│   ├── sse-proxy-integration-tracker.md
│   └── tasks/
│       ├── task-1.1-cli-sse-option.md
│       ├── task-1.2-sse-transport-wrapper.md
│       ├── task-2.1-dual-method-endpoint.md
│       ├── task-2.2-sse-response-handler.md
│       └── compatibility-2025-03-26.md
│
├── mcp-message-handling/           # MCP Protocol Understanding
│   ├── mcp-message-handling-tracker.md
│   ├── interceptor-mcp-spec.md
│   ├── recorder-mcp-spec.md
│   └── replay-mcp-spec.md
│
├── integration-tasks/              # Glue Tasks
│   ├── foundation-tasks.md        # Shared foundation components
│   └── glue-tasks.md              # Integration connection points
│
└── mcp-compliance/                 # Historical Reference
    ├── compliance-tracker.md      # Original MCP compliance work
    └── implementation-notes/
```

## Quick Links

### 🚀 Active Work
- **[SSE Proxy Integration](proxy-sse-message-tracker.md)** - Current priority
- **[SSE Integration Tasks](sse-proxy-integration/tasks/)** - Implementation tasks

### ✅ Completed Work
- **[Transport Context Refactor](transport-context-refactor/transport-context-tracker.md)** - Successfully completed
- **[Refactor Progress](shadowcat/plans/transport-context-refactor/PROGRESS.md)** - Detailed notes

### Reference Documentation
- **[SSE Proxy Integration](sse-proxy-integration/sse-proxy-integration-tracker.md)** - SSE transport details
- **[MCP Message Handling](mcp-message-handling/mcp-message-handling-tracker.md)** - MCP protocol details
- **[Integration Coordination](integration-coordination.md)** - How components work together

### Specifications
- **[Interceptor Spec](mcp-message-handling/interceptor-mcp-spec.md)** - MCP-aware interception
- **[Recorder Spec](mcp-message-handling/recorder-mcp-spec.md)** - Session recording
- **[Replay Spec](mcp-message-handling/replay-mcp-spec.md)** - Session replay

## Development Workflow

### For SSE Integration (Current Focus)
1. **Review the SSE proxy tracker** for implementation phases
2. **Start with Phase 1** - CLI and transport wrapper
3. **Use the new MessageEnvelope system** - TransportContext::sse() is ready
4. **Test with real SSE servers** - Ensure compatibility
5. **Update tracker** with progress

### Key Resources from Refactor
- `src/transport/envelope.rs` - MessageEnvelope and TransportContext
- `TransportContext::sse()` - Ready for SSE metadata (event ID, type, retry)
- Clean architecture with no technical debt

## Architecture Goals

### Near Term (Phases 1-3) - SSE Foundation
- ✅ Transport context separation (COMPLETE via refactor)
- 🚀 SSE transport in forward/reverse proxy
- 🚀 MCP message parsing over SSE
- 🚀 Request-response correlation

### Mid Term (Phases 4-5) - Enhanced Features
- Method-based interception rules
- Session recording with SSE context
- Storage and search capabilities

### Long Term (Phases 6-7) - Advanced Capabilities
- Intelligent replay with transformations
- Full MCP 2025-03-26 and 2025-06-18 support
- < 5% performance overhead

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Transport Refactor | Complete | ✅ Done (17.5 hours) |
| SSE Integration | 120-140 hours | 🚀 Ready to start |
| Latency Overhead | < 5% | To measure |
| Memory Usage | < 100MB/1000 sessions | To monitor |
| Throughput | > 10,000 msg/sec | To test |
| Test Coverage | > 90% | Ongoing |

## Getting Started with SSE Integration

```bash
# 1. Review the SSE proxy tracker
cat plans/proxy-sse-message-tracker.md

# 2. Check the new TransportContext capabilities
cat shadowcat/src/transport/envelope.rs

# 3. Start with Phase 1 tasks
cat plans/sse-proxy-integration/tasks/task-1.1-cli-sse-option.md
cat plans/sse-proxy-integration/tasks/task-1.2-sse-transport-wrapper.md

# 4. The foundation is ready - TransportContext::sse() supports:
# - Event ID tracking
# - Event type support  
# - Retry timing
# - Last-Event-ID handling
```

## Recent Achievements

### Transport Context Refactor (2025-08-08)
- **Completed in record time**: 17.5 hours vs 60 hour estimate
- **Clean architecture**: MessageEnvelope system replacing Frame
- **Zero technical debt**: All tests passing, clippy-clean
- **Ready for SSE**: TransportContext properly handles transport metadata

## Contact

For questions about the plan structure or implementation approach, refer to:
- Technical specifications in each plan directory
- Integration coordination document for cross-cutting concerns
- Transport refactor notes for architecture decisions

---

*Last Updated: 2025-08-08 - Transport Context Refactor Complete, SSE Integration Ready*