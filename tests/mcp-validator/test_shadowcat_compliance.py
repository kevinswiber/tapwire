#!/usr/bin/env python3
"""
Simple compliance test for Shadowcat proxy.
Tests basic MCP operations through the proxy.
"""

import json
import requests
from typing import Dict, Any

# Configuration
PROXY_URL = "http://localhost:8089/mcp"
# AUTH_TOKEN is handled by the proxy via env var or file, not passed from client

def test_initialize() -> Dict[str, Any]:
    """Test initialization."""
    print("Testing: Initialize...")
    
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "clientInfo": {"name": "compliance-test", "version": "1.0"},
            "capabilities": {},
            "clientCapabilities": {"protocol_versions": ["2025-03-26"]}
        }
    }
    
    response = requests.post(
        PROXY_URL,
        json=request,
        headers={
            "Content-Type": "application/json"
        }
    )
    
    result = response.json()
    
    if "result" in result:
        session_id = result["result"].get("sessionId")
        print(f"✅ Initialize: Success (Session: {session_id})")
        return {"status": "pass", "session_id": session_id}
    else:
        print(f"❌ Initialize: Failed - {result.get('error', 'Unknown error')}")
        return {"status": "fail", "error": result.get("error")}

def test_tools_list(session_id: str) -> Dict[str, Any]:
    """Test tools/list."""
    print("Testing: Tools List...")
    
    request = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    }
    
    response = requests.post(
        PROXY_URL,
        json=request,
        headers={
            "Content-Type": "application/json",
            "Mcp-Session-Id": session_id
        }
    )
    
    result = response.json()
    
    if "result" in result:
        tools = result["result"].get("tools", [])
        print(f"✅ Tools List: {len(tools)} tools found")
        for tool in tools:
            print(f"   - {tool['name']}: {tool.get('description', 'No description')}")
        return {"status": "pass", "tools": tools}
    else:
        print(f"❌ Tools List: Failed - {result.get('error', 'Unknown error')}")
        return {"status": "fail", "error": result.get("error")}

def test_tool_call(session_id: str) -> Dict[str, Any]:
    """Test calling a tool."""
    print("Testing: Tool Call (echo)...")
    
    request = {
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {"message": "Hello from Shadowcat compliance test!"}
        }
    }
    
    response = requests.post(
        PROXY_URL,
        json=request,
        headers={
            "Content-Type": "application/json",
            "Mcp-Session-Id": session_id
        }
    )
    
    result = response.json()
    
    if "result" in result:
        text = result["result"].get("text", "")
        print(f"✅ Tool Call: Success - Response: {text}")
        return {"status": "pass", "response": text}
    else:
        print(f"❌ Tool Call: Failed - {result.get('error', 'Unknown error')}")
        return {"status": "fail", "error": result.get("error")}

def test_ping(session_id: str) -> Dict[str, Any]:
    """Test ping."""
    print("Testing: Ping...")
    
    request = {
        "jsonrpc": "2.0",
        "id": 4,
        "method": "ping",
        "params": {}
    }
    
    response = requests.post(
        PROXY_URL,
        json=request,
        headers={
            "Content-Type": "application/json",
            "Mcp-Session-Id": session_id
        }
    )
    
    result = response.json()
    
    if "result" in result:
        print(f"✅ Ping: Success")
        return {"status": "pass"}
    else:
        print(f"❌ Ping: Failed - {result.get('error', 'Unknown error')}")
        return {"status": "fail", "error": result.get("error")}

def test_error_handling(session_id: str) -> Dict[str, Any]:
    """Test error handling."""
    print("Testing: Error Handling...")
    
    # Send request with invalid method (include id to get error response)
    request = {
        "jsonrpc": "2.0",
        "id": 999,
        "method": "invalid_method",
        "params": {}
    }
    
    response = requests.post(
        PROXY_URL,
        json=request,
        headers={
            "Content-Type": "application/json",
            "Mcp-Session-Id": session_id
        }
    )
    
    result = response.json()
    
    if "error" in result:
        error_code = result["error"].get("code")
        print(f"✅ Error Handling: Correctly returned error (code: {error_code})")
        return {"status": "pass", "error_code": error_code}
    else:
        print(f"❌ Error Handling: Should have returned an error")
        return {"status": "fail"}

def main():
    print("=" * 60)
    print("Shadowcat MCP Compliance Test")
    print("=" * 60)
    print(f"Testing proxy at: {PROXY_URL}")
    print()
    
    # Test 1: Initialize
    init_result = test_initialize()
    if init_result["status"] != "pass":
        print("\nCannot continue without successful initialization")
        return
    
    session_id = init_result["session_id"]
    print(f"\nUsing session: {session_id}\n")
    
    # Test 2: Tools List
    test_tools_list(session_id)
    print()
    
    # Test 3: Tool Call
    test_tool_call(session_id)
    print()
    
    # Test 4: Ping
    test_ping(session_id)
    print()
    
    # Test 5: Error Handling
    test_error_handling(session_id)
    
    print("\n" + "=" * 60)
    print("Compliance Test Complete")
    print("=" * 60)

if __name__ == "__main__":
    main()