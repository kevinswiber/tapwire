# MCP Validator Quick Start Guide

## Prerequisites

Make sure you have:
- Python 3.8+ installed
- `uv` for Python package management (or use pip/venv)

## Step 1: Set Up Python Environment

```bash
# Navigate to the validator directory
cd tools/mcp-validator

# Create a virtual environment using uv
uv venv

# Activate the environment
source .venv/bin/activate      # For bash/zsh
# OR
source .venv/bin/activate.fish  # For fish shell

# Install dependencies
uv pip install -r requirements.txt
uv pip install fastapi uvicorn  # Additional deps for HTTP server
```

## Step 2: Start the HTTP Test Server

### Basic Start
```bash
# From tools/mcp-validator directory with venv activated
python ref_http_server/reference_mcp_server.py --port 8088
```

### Run in Background
```bash
# Start in background
python ref_http_server/reference_mcp_server.py --port 8088 &

# Or with output to file
python ref_http_server/reference_mcp_server.py --port 8088 > server.log 2>&1 &
```

The server will start on `http://localhost:8088` with:
- **Main endpoint**: `POST /mcp` - Handles MCP protocol messages
- **Info endpoint**: `GET /` - Returns server info
- **WebSocket**: `WS /ws` - For WebSocket connections
- **SSE**: `GET /sse` - For Server-Sent Events

## Step 3: Test the Server

### Check if Running
```bash
# Get server info
curl http://localhost:8088/ | jq .
```

### Send a Test Request
```bash
# Initialize session (corrected parameters)
curl -X POST http://localhost:8088/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer valid-test-token-123" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2025-03-26",
      "clientInfo": {"name": "test-client", "version": "1.0"},
      "capabilities": {},
      "clientCapabilities": {"protocol_versions": ["2025-03-26"]}
    }
  }' | jq .
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "sessionId": "...",
    "protocolVersion": "2025-03-26",
    "serverInfo": {
      "name": "MCP Reference Server",
      "version": "1.0.0"
    },
    "serverCapabilities": {
      "protocolVersions": ["2024-11-05", "2025-03-26", "2025-06-18"],
      ...
    }
  }
}
```

### Test Tool Invocation
After initializing, use the session ID to call tools:
```bash
# Replace <SESSION_ID> with the sessionId from initialize response
curl -X POST http://localhost:8088/mcp \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer valid-test-token-123" \
  -H "Mcp-Session-Id: <SESSION_ID>" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "echo",
      "arguments": {"message": "Hello MCP!"}
    }
  }' | jq .
```

## Step 4: Stop the Server

```bash
# If running in foreground
Ctrl+C

# If running in background, find the process
ps aux | grep reference_mcp_server
# Then kill it
kill <PID>

# Or kill all Python MCP servers
pkill -f reference_mcp_server
```

## Available Test Tokens

The server accepts these authorization tokens:
- `valid-test-token-123` (hardcoded for testing)

To disable authentication:
```bash
export MCP_OAUTH_ENABLED=false
python ref_http_server/reference_mcp_server.py --port 8088
```

## Troubleshooting

### Server won't start
- Check if port 8088 is already in use: `lsof -i :8088`
- Kill existing process: `kill $(lsof -t -i:8088)`

### Import errors
- Make sure virtual environment is activated
- Reinstall dependencies: `uv pip install -r requirements.txt fastapi uvicorn`

### Authentication errors
- Use the correct token: `Bearer valid-test-token-123`
- Or disable auth with `export MCP_OAUTH_ENABLED=false`

### Wrong parameters error
Make sure to include all required fields:
- `protocolVersion` (not `protocol_version`)
- `clientInfo`
- `capabilities`
- `clientCapabilities` with `protocol_versions` array

## All-in-One Start Script

Create a script `start-test-server.sh`:
```bash
#!/bin/bash
cd "$(dirname "$0")"
source .venv/bin/activate
echo "Starting MCP Reference Server on port 8088..."
python ref_http_server/reference_mcp_server.py --port 8088
```

Make it executable:
```bash
chmod +x start-test-server.sh
./start-test-server.sh
```

## Testing with Shadowcat

Once the server is running, you can test Shadowcat as a reverse proxy:
```bash
# Start Shadowcat reverse proxy (from shadowcat directory)
# IMPORTANT: --upstream must include the full path to the endpoint
./target/release/shadowcat reverse \
  --bind 127.0.0.1:8089 \
  --upstream http://localhost:8088/mcp

# Test through the proxy - this works! ✅
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
      "capabilities": {},
      "clientCapabilities": {"protocol_versions": ["2025-03-26"]}
    }
  }' | jq .
```