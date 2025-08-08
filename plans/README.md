# Shadowcat Development Plans

## Active Development

### 🚨 Critical Prerequisite: Transport Context Refactor
**[Transport Context Refactor](transport-context-refactor/transport-context-tracker.md)**  
*This refactor must be completed before SSE integration can proceed. It addresses fundamental architectural issues with TransportMessage that block SSE proxy integration.*

Status: **In Progress** | Duration: **30-40 hours** | Phases: **5**

### Current Focus: Phase 0 - Analysis and Design
Analyzing TransportMessage usage and designing the MessageEnvelope system:
- **A.0**: MCP Protocol Specification Analysis (2h)
- **A.1**: TransportMessage Usage Analysis (3h)  
- **A.2**: Design MessageEnvelope Structure (2h)
- **A.3**: Create Migration Strategy (2h)
- **A.4**: Document Breaking Changes (1h)

### ⏸️ Deferred: SSE Proxy Integration
**[Unified Proxy-SSE-Message Tracker](proxy-sse-message-tracker.md)**  
*Implementation deferred pending completion of Transport Context Refactor.*

Status: **Blocked on Transport Refactor** | Duration: **120-140 hours** | Phases: **7**

**Reason for Deferral**: The current `TransportMessage` enum conflates transport, protocol, and JSON-RPC layers. SSE integration requires proper separation of transport metadata (event IDs, retry hints) from protocol messages. The refactor will introduce `MessageEnvelope` to properly handle this separation.

## Plan Structure

```
plans/
├── README.md (this file)
├── proxy-sse-message-tracker.md   # ⏸️ DEFERRED (blocked on refactor)
├── integration-coordination.md     # How SSE and MCP work together
│
├── transport-context-refactor/     # 🚨 ACTIVE - Critical Prerequisite
│   ├── transport-context-tracker.md # Main refactor tracker
│   └── tasks/                      # Phase 0 analysis tasks
│       ├── A.0-mcp-protocol-analysis.md
│       ├── A.1-transport-message-usage-analysis.md
│       ├── A.2-design-message-envelope.md
│       ├── A.3-create-migration-strategy.md
│       └── A.4-document-breaking-changes.md
│
├── sse-proxy-integration/          # SSE Transport Implementation (blocked)
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

### 🚨 Active Work
- **[Transport Context Refactor](transport-context-refactor/transport-context-tracker.md)** - Current priority
- **[Phase 0 Tasks](transport-context-refactor/tasks/)** - Analysis tasks to complete first

### ⏸️ Deferred Execution
- **[Unified Tracker](proxy-sse-message-tracker.md)** - Blocked on refactor
- **[Foundation Tasks](integration-tasks/foundation-tasks.md)** - Phase 0 components (deferred)
- **[Glue Tasks](integration-tasks/glue-tasks.md)** - Integration points (deferred)

### Reference Documentation
- **[SSE Proxy Integration](sse-proxy-integration/sse-proxy-integration-tracker.md)** - SSE transport details
- **[MCP Message Handling](mcp-message-handling/mcp-message-handling-tracker.md)** - MCP protocol details
- **[Integration Coordination](integration-coordination.md)** - How components work together

### Specifications
- **[Interceptor Spec](mcp-message-handling/interceptor-mcp-spec.md)** - MCP-aware interception
- **[Recorder Spec](mcp-message-handling/recorder-mcp-spec.md)** - Session recording
- **[Replay Spec](mcp-message-handling/replay-mcp-spec.md)** - Session replay

## Development Workflow

1. **Check the transport context refactor tracker** for current phase and tasks
2. **Complete Phase 0 analysis** before any implementation
3. **Pick a task** from the current phase (A.0 through A.4)
4. **Analyze thoroughly** following the task specifications
5. **Document findings** in the specified locations
6. **Update tracker** with completion status
7. **After refactor completion**, resume SSE proxy integration work

## Architecture Goals

### Near Term (Phases 0-3)
- ✅ SSE transport in forward/reverse proxy
- ✅ MCP message parsing and understanding
- ✅ Request-response correlation

### Mid Term (Phases 4-5)
- ✅ Method-based interception rules
- ✅ Session recording with context
- ✅ Storage and search capabilities

### Long Term (Phases 6-7)
- ✅ Intelligent replay with transformations
- ✅ Full MCP 2025-03-26 and 2025-06-18 support
- ✅ < 5% performance overhead

## Success Metrics

| Metric | Target | Tracking |
|--------|--------|----------|
| Latency Overhead | < 5% | Benchmark in Phase 7 |
| Memory Usage | < 100MB/1000 sessions | Monitor in Phase 5 |
| Throughput | > 10,000 msg/sec | Test in Phase 7 |
| Test Coverage | > 90% | Measure throughout |

## Getting Started

```bash
# 1. Review the transport context refactor tracker
cat plans/transport-context-refactor/transport-context-tracker.md

# 2. Start with Phase 0 analysis tasks
cat plans/transport-context-refactor/tasks/A.0-mcp-protocol-analysis.md
cat plans/transport-context-refactor/tasks/A.1-transport-message-usage-analysis.md

# 3. Understand the scope of changes
cd shadowcat
rg "TransportMessage" --type rust -l | wc -l  # 90 files affected

# 4. Begin analysis
# Start with A.0: MCP Protocol Specification Analysis
# Then A.1: TransportMessage Usage Analysis
```

## Contact

For questions about the plan structure or implementation approach, refer to:
- Technical specifications in each plan directory
- Integration coordination document for cross-cutting concerns
- Original MCP compliance tracker for historical context

---

*Last Updated: 2025-08-08*