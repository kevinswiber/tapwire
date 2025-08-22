#!/usr/bin/env python3
"""
Quick test to verify Shadowcat correctly proxies MCP messages.
"""

import json
import subprocess
import sys
import time
from pathlib import Path

def test_direct_server():
    """Test the reference server directly first."""
    print("Testing reference server directly...")
    
    # Start the reference server
    server = subprocess.Popen(
        ["python", "ref_stdio_server/stdio_server_2025_03_26.py"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Send initialize request
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "clientInfo": {"name": "test-client", "version": "1.0"},
            "capabilities": {}
        }
    }
    
    server.stdin.write(json.dumps(request) + "\n")
    server.stdin.flush()
    
    # Read response
    response_line = server.stdout.readline()
    if response_line:
        response = json.loads(response_line)
        print(f"Direct response: {json.dumps(response, indent=2)}")
        assert response.get("result") is not None, "Expected result in response"
        print("✅ Direct server test passed!")
    else:
        print("❌ No response from server")
        
    server.terminate()
    server.wait()

def test_shadowcat_proxy():
    """Test through Shadowcat forward proxy."""
    print("\nTesting through Shadowcat proxy...")
    
    shadowcat_path = Path("../../shadowcat/target/release/shadowcat")
    if not shadowcat_path.exists():
        print(f"❌ Shadowcat not found at {shadowcat_path}")
        return False
    
    # Start Shadowcat as forward proxy
    proxy = subprocess.Popen(
        [
            str(shadowcat_path),
            "--log-level", "error",
            "forward",
            "stdio",
            "--",
            "python", str(Path("ref_stdio_server/stdio_server_2025_03_26.py").absolute())
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Give it a moment to start
    time.sleep(1.0)
    
    # Check if proxy started OK
    if proxy.poll() is not None:
        print("❌ Proxy exited immediately")
        stderr = proxy.stderr.read()
        if stderr:
            print(f"Stderr: {stderr}")
        return False
    
    # Send initialize request through proxy
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "clientInfo": {"name": "test-client", "version": "1.0"},
            "capabilities": {}
        }
    }
    
    proxy.stdin.write(json.dumps(request) + "\n")
    proxy.stdin.flush()
    
    # Read response
    response_line = proxy.stdout.readline()
    if response_line:
        print(f"Raw response: {repr(response_line)}")
        if not response_line.strip():
            print("❌ Empty response from proxy")
            proxy.terminate()
            proxy.wait()
            return False
        response = json.loads(response_line)
        print(f"Proxy response: {json.dumps(response, indent=2)}")
        assert response.get("result") is not None, "Expected result in response"
        print("✅ Shadowcat proxy test passed!")
        success = True
    else:
        print("❌ No response through proxy")
        # Check stderr for errors
        stderr = proxy.stderr.read()
        if stderr:
            print(f"Proxy stderr: {stderr}")
        success = False
        
    proxy.terminate()
    proxy.wait()
    return success

def test_tool_invocation():
    """Test tool invocation through proxy."""
    print("\nTesting tool invocation through Shadowcat...")
    
    shadowcat_path = Path("../../shadowcat/target/release/shadowcat")
    
    # Start Shadowcat as forward proxy
    proxy = subprocess.Popen(
        [
            str(shadowcat_path),
            "--log-level", "error",
            "forward",
            "stdio",
            "--",
            "python", str(Path("ref_stdio_server/stdio_server_2025_03_26.py").absolute())
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    time.sleep(0.5)
    
    # Initialize first
    init_request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "clientInfo": {"name": "test-client", "version": "1.0"},
            "capabilities": {}
        }
    }
    
    proxy.stdin.write(json.dumps(init_request) + "\n")
    proxy.stdin.flush()
    init_response = proxy.stdout.readline()
    print(f"Init response: {init_response[:100]}...")
    
    # Now test echo tool
    tool_request = {
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "echo",
            "arguments": {"message": "Hello from Shadowcat!"}
        }
    }
    
    proxy.stdin.write(json.dumps(tool_request) + "\n")
    proxy.stdin.flush()
    
    tool_response_line = proxy.stdout.readline()
    if tool_response_line:
        tool_response = json.loads(tool_response_line)
        print(f"Tool response: {json.dumps(tool_response, indent=2)}")
        
        result = tool_response.get("result")
        if result and result.get("text") == "Hello from Shadowcat!":
            print("✅ Tool invocation test passed!")
            success = True
        else:
            print("❌ Tool response incorrect")
            success = False
    else:
        print("❌ No tool response")
        success = False
    
    proxy.terminate()
    proxy.wait()
    return success

if __name__ == "__main__":
    print("=" * 60)
    print("Testing Shadowcat MCP Proxy Capabilities")
    print("=" * 60)
    
    # Test 1: Direct server
    test_direct_server()
    
    # Test 2: Through proxy
    proxy_ok = test_shadowcat_proxy()
    
    # Test 3: Tool invocation
    if proxy_ok:
        test_tool_invocation()
    
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)