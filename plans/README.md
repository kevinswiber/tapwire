# Shadowcat Development Plans

> **📋 Creating a New Plan?** Follow the instructions in [plans/template/README.md](template/README.md) to create properly structured plans using our standard templates.

## Active Development

### 🎯 Currently Active Plans

The following plans are currently being worked on or are ready for implementation:

#### 🔥 **IMMEDIATE PRIORITY** - Critical Infrastructure
| Plan | Status | Estimated Duration | Description |
|------|--------|-------------------|-------------|
| **[Refactor Legacy Reverse Proxy](refactor-legacy-reverse-proxy/refactor-legacy-reverse-proxy-tracker.md)** | 🟢 **ACTIVE** | 25-35 hours | Extract 3,465-line monolith into clean modules |
| **[Reverse Proxy Session Mapping](reverse-proxy-session-mapping/reverse-proxy-session-mapping-tracker.md)** | 🟡 **Ready** | 8-12 hours | Dual session ID tracking for better routing |

#### ✅ **RECENTLY COMPLETED** 
| Plan | Status | Completion Date | Description |
|------|--------|----------------|-------------|
| **[Error Boundary Fix](archive/completed-2025-08-22-error-boundary-fix/error-fix-tracker.md)** | ✅ **COMPLETE** | 2025-08-22 | Fixed all module boundary violations (zero crate::Error refs remaining!) |
| **[Reverse Proxy Refactor](reverse-proxy-refactor/reverse-proxy-refactor-tracker.md)** | ✅ **COMPLETE** | 2025-08-18 | SSE resilience with EventTracker integration |
| **[Event Tracking Refactor](refactor-event-tracking/refactor-event-tracking-tracker.md)** | ✅ **COMPLETE** | 2025-08-18 | Consolidated event tracking with lazy persistence |

#### ⏸️ **PAUSED** - Requires Architecture Changes
| Plan | Status | Estimated Duration | Description |
|------|--------|-------------------|-------------|
| **[Multi-Session Forward Proxy](multi-session-forward-proxy/multi-session-forward-proxy-tracker.md)** | 🔴 **PAUSED** | 20-30 hours (revised) | Requires transport layer redesign - see [lessons learned](multi-session-forward-proxy/lessons-learned.md) |

#### Other Active Plans
| Plan | Status | Estimated Duration | Description |
|------|--------|-------------------|-------------|
| **[Config Validation Enhancement](config-validation-enhancement/config-validation-enhancement-tracker.md)** | 🟡 **Planning** | 16-24 hours | Rich error types, workload defaults, and user guidance |
| **[Traffic Recording Refactor](traffic-recording/traffic-recording-tracker.md)** | Planning | 16-24 hours | Fix SSE metadata handling and remove TransportContext::Sse |
| **[Better CLI Interface](better-cli-interface/better-cli-interface-tracker.md)** | Planning | 16-24 hours | Smart transport detection and improved UX |
| **[Full Batch Support](full-batch-support/full-batch-support-tracker.md)** | Analysis | 20-30 hours | Complete MCP batch message support |
| **[LLM Help Documentation](llm-help-documentation/llm-help-documentation-tracker.md)** | ✅ **COMPLETE** | 8-10 hours | Built-in help command with LLM-friendly output |
| **[Redis Session Storage](redis-session-storage/redis-storage-tracker.md)** | Design | 30-40 hours | Distributed session storage backend |
| **[Tape Format JSON Lines](tape-format-json-lines/tape-format-json-lines-tracker.md)** | **Phase 1 Complete** ✅ | 16-24 hours | JSONL tape format for streaming |
| **[Wassette Integration](wassette-integration/wassette-tracker.md)** | Phase C | 40-50 hours | WebAssembly module integration |

### 📌 Recommended Execution Order

Based on scope, dependencies, and conflict analysis, here's the optimal approach for tackling these plans:

#### **🔥 IMMEDIATE PRIORITY: Session Mapping Enhancement**

1. **[Reverse Proxy Session Mapping](reverse-proxy-session-mapping/)** (8-12 hours)
   - Dual session ID tracking for better routing
   - Builds on completed SSE resilience work
   - Improves debugging and monitoring capabilities
   - Improves debugging and monitoring capabilities

#### **Phase 1: Quick Win (Week 1, Days 1-2)** ✅ COMPLETE
**~~Start with: [LLM Help Documentation](llm-help-documentation/llm-help-documentation-tracker.md)~~** (8-10 hours)
- ✅ Smallest scope, self-contained
- ✅ No conflicts with other work
- ✅ Immediately useful for development
- ✅ Very low risk - purely additive feature
- ✅ **COMPLETED 2025-08-15**

#### **Phase 2: UX Enhancement (Week 1, Days 3-5 + Week 2)**  
**Then: [Better CLI Interface](better-cli-interface/better-cli-interface-tracker.md)** (16-24 hours)
- ✅ Isolated to CLI layer
- ✅ Improves developer experience
- ✅ Maintains backward compatibility
- ⚠️ Complete before adding new commands

#### **Phase 3: Parallel Work (Can start anytime)**

These can be worked on simultaneously without conflicts:

**[Tape Format JSON Lines](tape-format-json-lines/tape-format-tracker.md)** (16-24 hours)
- ✅ Completely independent subsystem (tape storage)
- ✅ No conflicts with CLI or protocol changes
- ✅ Major performance improvement for recording/replay

**[Full Batch Support](full-batch-support/full-batch-support-tracker.md)** (20-30 hours)
- ✅ Deep protocol/proxy layer changes
- ✅ Separate from UI and storage
- ⚠️ Start with analysis phase to determine value

### 🔄 Conflict Analysis

**Can be done in parallel:**
- ✅ `llm-help-documentation` + `tape-format-json-lines`
- ✅ `tape-format-json-lines` + `full-batch-support`  
- ✅ `better-cli-interface` + `tape-format-json-lines`

**Should be sequential:**
- ⚠️ `llm-help-documentation` → `better-cli-interface` (help needs updating after CLI changes)
- ⚠️ Complete `better-cli-interface` before features adding new commands

### ✅ Recently Completed

#### LLM Help Documentation - Complete (2025-08-15)
**[LLM Help Documentation](llm-help-documentation/llm-help-documentation-tracker.md)**  
*Successfully implemented in single session with full testing and documentation*

Added comprehensive CLI documentation generation for LLM consumption:
- ✅ Runtime generation using Clap introspection APIs
- ✅ Three output formats: Markdown, JSON, and Manpage
- ✅ Complete command tree with all options and arguments
- ✅ 8 integration tests + 2 unit tests all passing
- ✅ < 50ms generation time with zero runtime overhead

#### Transport Advanced Features - ProcessManager Integration (2025-08-14)
**[Transport Advanced Features](archive/transport-advanced-features/transport-advanced-features-tracker.md)**  
*Phase 1 ProcessManager integration complete with comprehensive design*

Successfully designed and documented the ProcessManager integration:
- ✅ Complete architectural design for subprocess lifecycle management
- ✅ Integration points identified across transport and proxy layers
- ✅ Performance profiling completed for SSE transport
- ✅ Ready for implementation phase

#### Transport Layer Refactor - Complete (2025-08-13)
**[Transport Layer Refactor](archive/transport-refactor/transport-refactor-tracker.md)**  
*Successfully completed with raw transport implementations and builder patterns*

Achieved clearer transport abstractions and unified protocol support:
- ✅ Raw transport layer for stdio, HTTP, and SSE
- ✅ Streamable HTTP support (HTTP POST + SSE)
- ✅ Builder pattern consistency across all transports
- ✅ Clean separation of transport mechanics from protocol logic

#### SSE/MCP Integration - Phase 8 COMPLETE! (2025-08-13)
**[Unified Proxy-SSE-Message Tracker](archive/proxy-sse-message-tracker.md)**  
*Successfully completed all 8 phases with comprehensive testing and documentation*

Achieved full SSE proxy integration with MCP message handling capabilities:
- ✅ All 163+ tests passing with zero clippy warnings
- ✅ Performance targets met (< 5% latency overhead)
- ✅ Complete API documentation and user guides
- ✅ Ready for production release (v0.2.0)

#### CLI Optimization - Phase C COMPLETE! (2025-08-12)
**[CLI Optimization Tracker](archive/cli-refactor-optimization/cli-optimization-tracker.md)**  
*Successfully completed with 802 tests passing and Grade A code quality*

Transformed Shadowcat from CLI-only to a robust, production-ready library:
- ✅ Library-first architecture with builder patterns
- ✅ Shell completions for all major shells
- ✅ Performance exceeding all targets (63,000+ sessions/second)
- ✅ Comprehensive telemetry and configuration support

#### Transport Context Refactor - COMPLETE! (2025-08-08)
**[Transport Context Refactor](archive/transport-context-refactor/transport-context-tracker.md)**  
*Successfully completed in 17.5 hours (71% reduction from 60 hour estimate)*

The refactor successfully separated protocol concerns from transport metadata, introducing the MessageEnvelope system that properly handles transport context.

## Plan Structure

```
plans/
├── README.md (this file)
├── template/                       # 📋 Templates for creating new plans
│   ├── README.md                  # Instructions for creating plans
│   ├── tracker.md                 # Main tracker template
│   ├── next-session-prompt.md     # Session setup template
│   └── task.md                    # Individual task template
│
├── refactor-legacy-reverse-proxy/  # 🔥 ACTIVE - Clean module extraction
├── reverse-proxy-session-mapping/  # 🔥 TOP PRIORITY - Dual session IDs
├── multi-session-forward-proxy/    # 🔥 TOP PRIORITY - Concurrent clients
├── better-cli-interface/           # 🎯 ACTIVE - Smart transport detection
├── full-batch-support/             # 🎯 ACTIVE - MCP batch messages
├── llm-help-documentation/         # ✅ COMPLETE - LLM-friendly help
├── redis-session-storage/          # 🎯 ACTIVE - Distributed storage
├── tape-format-json-lines/         # 🎯 ACTIVE - JSONL format
├── wassette-integration/           # 🎯 ACTIVE - WebAssembly modules
│
└── archive/                        # ✅ Completed and historical plans
    ├── transport-refactor/         # ✅ Raw transport layer complete
    ├── transport-advanced-features/ # ✅ ProcessManager design complete
    ├── tape-storage-providers/     # ✅ Storage abstraction design
    ├── proxy-sse-message-tracker.md
    ├── cli-refactor-optimization/
    ├── transport-context-refactor/
    ├── sse-proxy-integration/
    ├── mcp-message-handling/
    └── [other completed work]
```

## Quick Links

### 🎯 Active Work

#### 🔥 Top Priority - Critical Infrastructure
- **[Reverse Proxy Session Mapping](reverse-proxy-session-mapping/reverse-proxy-session-mapping-tracker.md)** - Dual session IDs for SSE/failover
- **[Multi-Session Forward Proxy](multi-session-forward-proxy/multi-session-forward-proxy-tracker.md)** - Concurrent client support

#### Other Active Plans
- **[Better CLI Interface](better-cli-interface/better-cli-interface-tracker.md)** - Smart transport detection
- **[Full Batch Support](full-batch-support/full-batch-support-tracker.md)** - MCP batch messages
- **[LLM Help Documentation](llm-help-documentation/llm-help-documentation-tracker.md)** - LLM-friendly help ✅
- **[Redis Session Storage](redis-session-storage/redis-storage-tracker.md)** - Distributed storage
- **[Tape Format JSON Lines](tape-format-json-lines/tape-format-json-lines-tracker.md)** - JSONL format
- **[Wassette Integration](wassette-integration/wassette-tracker.md)** - WebAssembly modules

### ✅ Completed Work
- **[Transport Refactor](archive/transport-refactor/transport-refactor-tracker.md)** - Raw transport layer complete
- **[Transport Advanced Features](archive/transport-advanced-features/transport-advanced-features-tracker.md)** - ProcessManager design
- **[Tape Storage Providers](archive/tape-storage-providers/tape-storage-providers-tracker.md)** - Storage abstraction
- **[SSE/MCP Integration](archive/proxy-sse-message-tracker.md)** - Phase 8 complete, production ready
- **[CLI Optimization](archive/cli-refactor-optimization/cli-optimization-tracker.md)** - Library architecture complete
- **[Transport Context Refactor](archive/transport-context-refactor/transport-context-tracker.md)** - MessageEnvelope system

### Reference Documentation (Archived)
- **[SSE Proxy Integration](archive/sse-proxy-integration/sse-proxy-integration-tracker.md)** - SSE transport details
- **[MCP Message Handling](archive/mcp-message-handling/mcp-message-handling-tracker.md)** - MCP protocol details
- **[Integration Coordination](archive/integration-coordination.md)** - How components work together

### Specifications (Archived)
- **[Interceptor Spec](archive/mcp-message-handling/interceptor-mcp-spec.md)** - MCP-aware interception
- **[Recorder Spec](archive/mcp-message-handling/recorder-mcp-spec.md)** - Session recording
- **[Replay Spec](archive/mcp-message-handling/replay-mcp-spec.md)** - Session replay

## Development Workflow

### Starting a New Feature
1. **Choose an active plan** from the list above
2. **Review the tracker** for current status and next steps
3. **Check the next-session-prompt.md** if available
4. **Follow the task structure** defined in the plan
5. **Update the tracker** with progress

### Key Active Features
- **Better CLI**: Smart transport detection and improved UX
- **Batch Support**: Complete MCP batch message handling
- **LLM Documentation**: Built-in help with LLM-friendly output
- **Redis Storage**: Distributed session management
- **JSONL Tapes**: Streaming-friendly tape format
- **Wassette**: WebAssembly module integration

## Architecture Goals

### Active Development Areas
- 🎯 **Better CLI UX**: Smart transport detection, improved error messages
- 🎯 **Complete MCP Support**: Full batch message handling
- 🎯 **Developer Experience**: LLM-friendly documentation and help
- 🎯 **Scalability**: Redis-backed distributed session storage
- 🎯 **Streaming**: JSONL tape format for real-time processing
- 🎯 **Extensibility**: WebAssembly module integration

### Completed Achievements
- ✅ Raw transport layer with unified abstractions
- ✅ ProcessManager design for subprocess lifecycle
- ✅ Storage provider abstraction architecture
- ✅ Transport context separation (MessageEnvelope system)
- ✅ SSE transport in forward/reverse proxy
- ✅ MCP message parsing and correlation
- ✅ Full MCP 2025-03-26 and 2025-06-18 support
- ✅ < 5% performance overhead achieved
- ✅ Library-first architecture with builder patterns
- ✅ Comprehensive testing (802+ tests)

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Transport Context Refactor | Complete | ✅ Done (17.5 hours) |
| Transport Layer Refactor | 40-50 hours | ✅ Complete |
| Transport Advanced Features | Design Phase | ✅ Complete |
| SSE Integration | 120-140 hours | ✅ Complete |
| CLI Optimization | 73 hours | ✅ Complete |
| Better CLI Interface | 20-30 hours | 🎯 In Progress |
| Full Batch Support | 15-25 hours | 🎯 Analysis |
| LLM Documentation | 10-15 hours | 🎯 Research |
| Redis Storage | 30-40 hours | 🎯 Design |
| Tape Format JSONL | 15-20 hours | 🎯 Planning |
| Wassette Integration | 40-50 hours | 🎯 Phase C |
| Latency Overhead | < 5% | ✅ Achieved |
| Memory Usage | < 100MB/1000 sessions | ✅ Verified |
| Throughput | > 10,000 msg/sec | ✅ Exceeded (63k+/sec) |
| Test Coverage | > 90% | ✅ 802+ tests |

## Getting Started with Active Features

```bash
# 1. List all active plan directories
ls -la plans/ | grep -E "better-cli|full-batch|llm-help|redis|tape-format|wassette"

# 2. Choose a feature to work on and review its tracker
cat plans/better-cli-interface/better-cli-interface-tracker.md
cat plans/full-batch-support/full-batch-support-tracker.md
# ... etc

# 3. Check for next session prompts
cat plans/*/next-session-prompt.md 2>/dev/null

# 4. Review task definitions
ls plans/*/tasks/

# 5. Start implementing based on tracker status
```

## Recent Achievements

### Transport Advanced Features (2025-08-14)
- **ProcessManager design complete**: Comprehensive subprocess lifecycle management
- **Integration points mapped**: Clear path for implementation across layers
- **SSE performance profiled**: Optimization opportunities identified
- **Ready for implementation**: Design documentation complete

### Transport Layer Refactor Complete (2025-08-13)
- **Raw transport abstractions**: Clean separation of transport mechanics
- **Streamable HTTP support**: Unified HTTP POST + SSE handling
- **Builder patterns**: Consistent construction across all transports
- **Protocol separation**: Transport layer free of protocol logic

### SSE/MCP Integration Complete (2025-08-13)
- **All 8 phases complete**: Comprehensive SSE proxy with MCP support
- **Performance validated**: < 5% latency overhead confirmed
- **Production ready**: v0.2.0 with full documentation
- **163+ tests**: All passing with zero warnings

### CLI Optimization Complete (2025-08-12)
- **Library architecture**: Builder patterns for all components
- **Performance**: 63,000+ sessions/second achieved
- **802 tests**: Comprehensive coverage
- **Shell completions**: All major shells supported


## Next Steps

1. **Choose an active plan** from the 6 currently in progress
2. **Review the tracker** to understand current status
3. **Pick up where previous work left off** using task files
4. **Update trackers** as you make progress

---

*Last Updated: 2025-08-18 - Created Refactor Legacy Reverse Proxy plan to extract 3,465-line monolith into clean modules*