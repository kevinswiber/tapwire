# Architecture Revision Summary

## Key Changes from Original Design

Based on feedback, the module architecture has been revised to:

1. **Avoid naming conflicts** with existing modules
2. **Remove admin UI entirely** (will be handled separately, possibly on different bind address)
3. **Simplify handler structure** (only one main /mcp handler, not multiple)
4. **Clarify SSE role** (streaming utility, not a handler)

## Revised Module Structure

### Removed Components (~900 lines)
- **Admin UI**: All admin-related handlers, HTML generation, and routes
- **Admin tests**: Related test code

### Renamed to Avoid Conflicts
- `proxy::reverse::transport/` → `proxy::reverse::upstream/`
  - Handles upstream server communication (stdio, HTTP)
  - Avoids conflict with main `transport` module
  
- `proxy::reverse::session/` → `proxy::reverse::session_ops.rs`
  - Single file for reverse proxy-specific session operations
  - Avoids conflict with main `session` module

### Simplified Architecture

#### Single Handler Approach
Instead of multiple handlers:
- **One main handler**: `mcp_handler.rs` for `/mcp` endpoint
- **SSE as utility**: `sse_streaming.rs` for response streaming (not a handler)
- **Health/metrics**: Simple functions in router, not separate handlers

#### Clear Module Purposes

| Module | Purpose | Avoids Conflict With |
|--------|---------|---------------------|
| `upstream/` | Process requests to upstream servers | `transport/` |
| `session_ops.rs` | Reverse proxy session operations | `session/` |
| `mcp_handler.rs` | Main /mcp request handler | N/A |
| `sse_streaming.rs` | SSE response streaming utility | N/A |

## File Count Reduction

**Before**: 3,682 lines
**After removing admin**: ~2,750 lines
**Modules**: 12 focused modules (down from 15)

## Benefits of Revision

1. **No naming confusion**: Clear distinction between reverse proxy modules and core modules
2. **Simpler mental model**: One handler, clear utilities
3. **Smaller codebase**: ~900 lines removed with admin UI
4. **Cleaner separation**: Admin functionality completely separate

## Migration Impact

### Easier Extractions
- Admin removal is just deletion (no extraction needed)
- Fewer modules to create
- Clearer boundaries

### Testing Changes
- Remove admin-related tests
- Keep all other tests unchanged
- Simpler test structure

## Next Steps

1. **Phase 1**: Remove admin UI first (simple deletion)
2. **Phase 2**: Extract configs, errors, metrics
3. **Phase 3**: Create upstream/ module for server processing
4. **Phase 4**: Extract SSE streaming utility
5. **Phase 5**: Refactor main handler