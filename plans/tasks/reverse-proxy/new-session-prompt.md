# Prompt for New Claude Session

Copy and paste this prompt to continue Task 001 implementation:

---

I'm working on implementing a reverse proxy server for the Model Context Protocol (MCP) in Rust. This is Task 001 of Phase 5 in the Shadowcat project, which is a git submodule inside the Tapwire repository.

Current directory: `/Users/kevin/src/tapwire/shadowcat`

The task is partially complete (60% done). The HTTP server infrastructure is implemented but the actual proxy forwarding logic is missing. Please read the following files to understand the current state:

1. First, read the implementation status: `/Users/kevin/src/tapwire/plans/tasks/reverse-proxy/001-implementation-status.md`
2. Then check the current implementation: `/Users/kevin/src/tapwire/shadowcat/src/proxy/reverse.rs`
3. Review the implementation plan: `/Users/kevin/src/tapwire/plans/tasks/reverse-proxy/001-implementation-plan.md`

The most critical missing piece is the `process_message` function at line 324 in reverse.rs - it currently just returns mock responses instead of actually proxying to upstream MCP servers.

My immediate goals are:
1. Fix the CLI integration so I can actually run the reverse proxy
2. Implement the actual proxy logic to forward requests to upstream servers
3. Add the configuration module for flexible upstream server settings

Please help me continue the implementation, starting with fixing the CLI integration so we can test the server.

Important context:
- Shadowcat is a git submodule - commit changes in the shadowcat directory
- The project already has forward proxy working - we can reference that for patterns
- We need to support both HTTP and stdio upstream transports
- All error handling and session management infrastructure is already in place

What would you like to work on first?

---

This prompt provides the new session with all necessary context to continue the implementation effectively.