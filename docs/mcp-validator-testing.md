# MCP Validator Testing Guide

## Overview

The MCP Validator (`tools/mcp-validator`) provides reference MCP server implementations and compliance testing tools. This guide explains how to use it to test Shadowcat's MCP protocol compliance.

## Setup

### 1. Initialize the Validator Submodule
```bash
# If not already cloned with --recursive
git submodule update --init --recursive

# Or add it fresh
git submodule add https://github.com/Janix-ai/mcp-validator.git tools/mcp-validator
```

### 2. Set Up Python Environment
```bash
cd tools/mcp-validator

# Create virtual environment with uv
uv venv

# Activate the environment
source .venv/bin/activate  # For bash/zsh
# Or: source .venv/bin/activate.fish  # For fish shell

# Install dependencies
uv pip install -r requirements.txt
uv pip install fastapi uvicorn  # For HTTP server
```

## Testing Shadowcat Reverse Proxy

### 1. Start the Reference HTTP Server
```bash
cd tools/mcp-validator
source .venv/bin/activate
python ref_http_server/reference_mcp_server.py --port 8088
```

The server will start on `http://localhost:8088` with:
- Main endpoint: `/mcp` (POST)
- Info endpoint: `/` (GET)
- OAuth 2.1 authentication support (optional)

### 2. Start Shadowcat Reverse Proxy
```bash
cd shadowcat
cargo build --release

# Start reverse proxy pointing to the reference server
# IMPORTANT: Include the full path in --upstream
./target/release/shadowcat reverse \
  --bind 127.0.0.1:8089 \
  --upstream http://localhost:8088/mcp
```

### 3. Test Basic MCP Request

#### Direct to Reference Server (baseline)
```bash
curl -X POST http://localhost:8088/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer valid-test-token-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-03-26",
      "clientInfo": {"name": "test", "version": "1.0"},
      "capabilities": {}
    }
  }' | jq .
```

#### Through Shadowcat Proxy
```bash
curl -X POST http://localhost:8089/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer valid-test-token-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-03-26",
      "clientInfo": {"name": "test", "version": "1.0"},
      "capabilities": {}
    }
  }' | jq .
```

### 4. Test Tool Invocation
After initialization, test tool calls:
```bash
# Echo tool test
curl -X POST http://localhost:8089/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer valid-test-token-123" \
  -H "Mcp-Session-Id: <session-id-from-init>" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello from Shadowcat!"}
    }
  }' | jq .
```

## Running Compliance Tests

### HTTP Compliance Test
```bash
cd tools/mcp-validator
source .venv/bin/activate

# Test the reference server directly
python mcp_testing/scripts/http_compliance_test.py \
  --server-url http://localhost:8088 \
  --debug

# Test through Shadowcat proxy
python mcp_testing/scripts/http_compliance_test.py \
  --server-url http://localhost:8089 \
  --debug
```

### Generate Compliance Report
```bash
python -m mcp_testing.scripts.compliance_report \
  --server-url http://localhost:8089 \
  --protocol-version 2025-06-18 \
  --output-format json > shadowcat-compliance.json
```

## Testing Different Protocol Versions

The validator supports multiple MCP protocol versions:
- `2024-11-05` - Original version
- `2025-03-26` - Adds async tools support
- `2025-06-18` - Adds structured tool output, OAuth 2.1

Test each version:
```bash
# Test each protocol version
for version in "2024-11-05" "2025-03-26" "2025-06-18"; do
  echo "Testing protocol version: $version"
  python -m mcp_testing.scripts.compliance_report \
    --server-url http://localhost:8089 \
    --protocol-version $version
done
```

## SSE (Server-Sent Events) Testing

For SSE transport testing:
```bash
# Start SSE connection through proxy
curl -N -H "Accept: text/event-stream" \
     -H "Authorization: Bearer valid-test-token-123" \
     http://localhost:8089/mcp/sse
```

## Authentication

### Default Authentication
By default, the reference server has OAuth enabled and requires authentication. It accepts the following test token:
- `valid-test-token-123`

To disable authentication:
```bash
export MCP_OAUTH_ENABLED=false
python ref_http_server/reference_mcp_server.py --port 8088
```

### OAuth 2.1 Authentication Testing

Enable OAuth in the reference server:
```bash
export MCP_OAUTH_ENABLED=true
export MCP_OAUTH_INTROSPECTION_URL=https://auth.example.com/oauth/introspect
export MCP_OAUTH_REQUIRED_SCOPES=mcp:read,mcp:write

python ref_http_server/reference_mcp_server.py --port 8088
```

Test authentication flow:
```bash
# Request without token (should get 401)
curl -X POST http://localhost:8089/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' \
  -v

# Check for WWW-Authenticate header in response
# Should contain: Bearer realm="mcp-server"
```

## Known Issues and Fixes

### Header Case Sensitivity
Some MCP server implementations incorrectly treat HTTP headers as case-sensitive. While HTTP headers are case-insensitive per RFC 7230, Shadowcat accommodates buggy servers:
- Fixed: Shadowcat now sends `Mcp-Session-Id` (mixed case) to match what some servers expect
- Fixed: Response headers properly use `HeaderName::from_static()` for consistent handling

## Debugging Tips

### 1. Check Server Logs
The reference server logs all requests:
```bash
# Run with debug logging
export LOG_LEVEL=DEBUG
python ref_http_server/reference_mcp_server.py --port 8088
```

### 2. Check Shadowcat Logs
```bash
# Run with trace-level logging
RUST_LOG=shadowcat=trace ./target/release/shadowcat reverse \
  --bind 127.0.0.1:8089 \
  --upstream http://localhost:8088/mcp
```

### 3. Common Issues

**404 Not Found**: The upstream URL must include the complete path to the endpoint
```bash
# ✅ CORRECT - Include the full path in --upstream
--upstream http://localhost:8088/mcp

# ❌ WRONG - These will result in 404 or 405 errors
--upstream http://localhost:8088        # Missing endpoint path
--upstream http://localhost:8088/       # Missing endpoint path
--upstream http://localhost:8088/messages  # Wrong endpoint
```

**Important**: Shadowcat doesn't append paths automatically. The `--upstream` parameter should contain the complete URL including the path where the MCP endpoint is hosted.

**Connection Refused**: Check that the reference server is running and accessible
```bash
# Test direct connection
curl http://localhost:8088/
```

**Auth Errors**: The reference server accepts these test tokens:
- `valid-test-token-123`
- Or disable auth: `export MCP_OAUTH_ENABLED=false`

## CI/CD Integration

Add to GitHub Actions:
```yaml
- name: Start MCP Validator Server
  run: |
    cd tools/mcp-validator
    python -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    python ref_http_server/reference_mcp_server.py --port 8088 &
    sleep 2

- name: Run Compliance Tests
  run: |
    ./target/release/shadowcat reverse \
      --bind 127.0.0.1:8089 \
      --upstream http://localhost:8088/mcp &
    sleep 1
    
    cd tools/mcp-validator
    source .venv/bin/activate
    python -m mcp_testing.scripts.compliance_report \
      --server-url http://localhost:8089 \
      --protocol-version 2025-06-18
```

## Next Steps

1. **Fix proxy issues**: Resolve 404 errors when proxying to `/mcp` endpoint
2. **Create test scripts**: Automate common test scenarios
3. **Add to E2E tests**: Integrate validator servers into test harness
4. **Monitor compliance**: Track protocol compliance over time
5. **Test recording/replay**: Verify tape recording with validator traffic

## References

- [MCP Validator GitHub](https://github.com/Janix-ai/mcp-validator)
- [MCP Specification](https://spec.modelcontextprotocol.io)
- [Shadowcat E2E Testing Plan](../plans/e2e-testing-framework/)