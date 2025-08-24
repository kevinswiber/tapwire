# Transport Implementation Deviation Analysis

## Current Implementation Status

We've implemented:
1. **StdioTransport** - Uses process's OWN stdin/stdout
2. **SubprocessTransport** - SPAWNS a subprocess and manages it
3. **HttpTransport** - HTTP with SSE support

## The Question: Do We Need SubprocessTransport?

### Use Case Analysis

#### MCP Server Scenarios
- **As stdio process**: Uses `StdioTransport` to read/write its own stdin/stdout
- **As HTTP server**: Uses `HttpTransport` in server mode
- **Never spawns subprocesses** - servers don't spawn other servers

#### MCP Client Scenarios
- **Connecting to HTTP server**: Uses `HttpTransport` 
- **Connecting to stdio server**: Two options:
  1. **With SubprocessTransport**: Client spawns the server process
  2. **Without SubprocessTransport**: External orchestration spawns both

#### Shadowcat Proxy Scenarios
- **Incoming**: Acts as server (HTTP/SSE)
- **Outgoing to stdio**: Needs to spawn and manage MCP server processes
- **Complex requirements**: Process pools, restart logic, monitoring

## The Core Insight

The user is correct that **our MCP library is not a proxy**. The key differences:

### What Shadowcat Needs (Proxy)
- Process lifecycle management
- Process pools for scalability
- Restart on failure
- Health monitoring
- Resource limits
- Complex subprocess orchestration

### What MCP Library Needs (Client/Server)
- **Server**: Just StdioTransport (read/write own stdin/stdout)
- **Client**: Debatable - does it need to spawn stdio servers?

## Recommendation: Simplify

### Option 1: Keep SubprocessTransport (Current)
**Pros:**
- MCP clients can spawn stdio servers directly
- Matches typical usage patterns (e.g., `npx @modelcontextprotocol/server-everything`)
- Self-contained library

**Cons:**
- Adds complexity we might not need
- Process management is arguably an application concern
- Duplicates shadowcat's subprocess logic

### Option 2: Remove SubprocessTransport (Simpler)
**Pros:**
- Cleaner separation of concerns
- MCP library focuses on protocol, not process management
- Applications (like shadowcat) handle subprocess orchestration
- Simpler to maintain and test

**Cons:**
- MCP clients can't directly connect to stdio servers
- Requires external orchestration
- Less convenient for simple use cases

## Architectural Decision Review

From `architectural-decisions.md`:
- Type-conscious naming: `stdio::Transport` not `StdioTransport` ✅
- Transport organization clearly defined
- But subprocess wasn't explicitly discussed in the architecture

The plan mentions extracting "stdio::Transport" but not "subprocess::Transport".

## What's Missing from StdioTransport?

Looking at our current `StdioTransport`, it appears complete for its purpose:
- ✅ Reads from process's stdin
- ✅ Writes to process's stdout  
- ✅ Line-delimited JSON format
- ✅ Message size limits
- ✅ Proper error handling

## Conclusion

**We should REMOVE SubprocessTransport from the MCP library** because:

1. **Not in original plan**: The architecture docs don't mention subprocess transport
2. **Separation of concerns**: Process management is an application/proxy concern
3. **Shadowcat already has it**: The proxy has sophisticated subprocess management in `src/transport/outgoing/subprocess.rs`
4. **Simplicity**: The MCP library should focus on protocol, not orchestration

### Proposed Structure
```
mcp::transport::
├── stdio::Transport      // For stdio servers (and stdio clients if needed)
├── http::Transport       // For HTTP clients and servers
└── http::streaming::sse  // SSE support for HTTP
```

### For Users Who Need Subprocess
Users who need to spawn stdio servers can:
1. Use shadowcat as a proxy (recommended)
2. Implement their own subprocess management
3. Use a separate process manager

This keeps the MCP library focused and simple while letting applications like shadowcat handle complex process orchestration.

## Action Items
1. Remove `subprocess.rs` from the MCP crate
2. Update documentation to clarify stdio usage
3. Ensure shadowcat's subprocess implementation remains separate
4. Update tracker to reflect this decision