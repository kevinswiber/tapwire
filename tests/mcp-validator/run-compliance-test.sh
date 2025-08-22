#!/bin/bash
# Run MCP compliance tests against Shadowcat

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR/../.."
VALIDATOR_DIR="$PROJECT_ROOT/tools/mcp-validator"

echo "Running MCP Validator Compliance Tests"
echo "======================================"

# Check if virtual environment exists
if [ ! -d "$VALIDATOR_DIR/.venv" ]; then
    echo "Setting up Python environment..."
    cd "$VALIDATOR_DIR"
    uv venv
    source .venv/bin/activate
    uv pip install -r requirements.txt
    uv pip install fastapi uvicorn
else
    echo "Using existing Python environment"
    cd "$VALIDATOR_DIR"
    source .venv/bin/activate
fi

# Check if reference server is running
if ! curl -s http://localhost:8088/ > /dev/null 2>&1; then
    echo "Starting MCP reference server..."
    python ref_http_server/reference_mcp_server.py --port 8088 &
    SERVER_PID=$!
    sleep 2
    echo "Server started (PID: $SERVER_PID)"
else
    echo "Reference server already running"
fi

# Check if Shadowcat proxy is running
if ! curl -s http://localhost:8089/ > /dev/null 2>&1; then
    echo "Starting Shadowcat reverse proxy..."
    cd "$PROJECT_ROOT/shadowcat"
    ./target/release/shadowcat reverse \
        --bind 127.0.0.1:8089 \
        --upstream http://localhost:8088/mcp &
    PROXY_PID=$!
    sleep 1
    echo "Proxy started (PID: $PROXY_PID)"
else
    echo "Shadowcat proxy already running"
fi

# Run our compliance test
echo ""
echo "Running compliance test..."
cd "$SCRIPT_DIR"
python test_shadowcat_compliance.py

# Cleanup (optional - uncomment to auto-cleanup)
# if [ ! -z "$SERVER_PID" ]; then
#     echo "Stopping reference server..."
#     kill $SERVER_PID 2>/dev/null || true
# fi
# if [ ! -z "$PROXY_PID" ]; then
#     echo "Stopping proxy..."
#     kill $PROXY_PID 2>/dev/null || true
# fi

echo ""
echo "Test complete!"