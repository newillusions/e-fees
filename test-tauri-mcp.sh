#!/bin/bash

echo "Testing tauri-mcp server..."
echo ""

# Test 1: Check binary exists
echo "1. Checking tauri-mcp binary..."
if [ -f "/Users/martin/.cargo/bin/tauri-mcp" ]; then
    echo "   ✓ Binary exists"
    /Users/martin/.cargo/bin/tauri-mcp --version
else
    echo "   ✗ Binary not found"
    exit 1
fi

# Test 2: Check app path
echo ""
echo "2. Checking app path..."
if [ -f "/Volumes/base/dev/e-fees/src-tauri/tauri.conf.json" ]; then
    echo "   ✓ Tauri config found at app path"
else
    echo "   ✗ Tauri config not found"
    exit 1
fi

# Test 3: Test server response
echo ""
echo "3. Testing server response..."
RESPONSE=$(echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | /Users/martin/.cargo/bin/tauri-mcp --app-path /Volumes/base/dev/e-fees serve 2>/dev/null | head -1)

if echo "$RESPONSE" | grep -q '"serverInfo"'; then
    echo "   ✓ Server responds correctly"
    echo "   Response: $(echo $RESPONSE | jq -r '.result.serverInfo.name') v$(echo $RESPONSE | jq -r '.result.serverInfo.version')"
else
    echo "   ✗ Server did not respond correctly"
    echo "   Response: $RESPONSE"
    exit 1
fi

echo ""
echo "All tests passed! The tauri-mcp server is working correctly."
echo ""
echo "To reconnect in Claude, you may need to:"
echo "1. Exit this Claude session"
echo "2. Start a new session with: claude-mcp"
echo "3. Or manually reconnect the MCP servers"