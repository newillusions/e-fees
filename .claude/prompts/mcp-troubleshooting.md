# MCP Troubleshooting Guide for E-Fees Project

## Custom MCP Integration
- **Socket Path**: `/tmp/tauri-mcp-e2e.sock`
- **Purpose**: End-to-end testing integration with Tauri v2 app
- **Protocol**: Unix domain socket communication

## Quick Diagnostic Checklist

### 1. Socket Connection Issues
```bash
# Check if socket exists
ls -l /tmp/tauri-mcp-e2e.sock

# Check socket permissions
stat /tmp/tauri-mcp-e2e.sock

# Test socket connectivity
nc -U /tmp/tauri-mcp-e2e.sock
```

**Common Issues:**
- Socket file doesn't exist → MCP server not running
- Permission denied → Check file ownership and permissions
- Connection refused → Server crashed or not listening

**Solutions:**
```bash
# Restart MCP server
pkill -f tauri-mcp-e2e
./start-mcp-server.sh

# Fix permissions
chmod 666 /tmp/tauri-mcp-e2e.sock

# Check if another process is using the socket
lsof /tmp/tauri-mcp-e2e.sock
```

### 2. MCP Server Not Starting

**Symptoms:**
- Socket file not created
- No process listening on socket
- Server logs showing errors

**Diagnostic Steps:**
```bash
# Check server logs
tail -f logs/mcp-server.log

# Verify dependencies
npm list | grep mcp
cargo tree | grep mcp

# Check port/socket conflicts
netstat -a | grep tauri-mcp
```

**Common Causes:**
1. **Port/socket already in use**
   - Solution: Kill existing process or use different socket path

2. **Missing dependencies**
   - Solution: Run `npm install` or `cargo build`

3. **Configuration errors**
   - Check: `.env` file, `mcp-config.json`
   - Validate: JSON syntax, required fields

### 3. Message Format Issues

**MCP Message Structure:**
```json
{
  "jsonrpc": "2.0",
  "id": "unique-id",
  "method": "test/action",
  "params": {
    "action": "click",
    "selector": "#button"
  }
}
```

**Common Errors:**
- Invalid JSON → Validate with `jq`
- Missing required fields → Check against schema
- Wrong method name → Refer to API documentation

### 4. Tauri Integration Issues

**E2E Test Communication Flow:**
```
Test Script → MCP Client → Unix Socket → MCP Server → Tauri App
```

**Debugging:**
```bash
# Enable verbose logging
export MCP_DEBUG=1
export TAURI_DEBUG=1

# Run test with debug output
npm run test:e2e -- --verbose

# Check Tauri app logs
tail -f src-tauri/target/debug/app.log
```

**Common Issues:**
1. **IPC timeout**
   - Increase timeout in test configuration
   - Check if Tauri app is responding

2. **State synchronization**
   - Verify MCP server is tracking app state correctly
   - Check for race conditions in event handlers

3. **Event propagation**
   - Ensure Tauri events are emitted correctly
   - Verify MCP server is subscribed to events

### 5. Performance Issues

**Symptoms:**
- Slow test execution
- Socket read/write timeouts
- Memory leaks

**Optimization:**
```bash
# Monitor socket performance
strace -e trace=network nc -U /tmp/tauri-mcp-e2e.sock

# Check memory usage
ps aux | grep mcp

# Profile socket I/O
perf record -e syscalls:sys_enter_sendto,syscalls:sys_enter_recvfrom -p $(pgrep mcp-server)
```

**Solutions:**
- Implement connection pooling
- Use message batching
- Add response caching
- Optimize message serialization

### 6. Error Response Codes

| Code | Meaning | Action |
|------|---------|--------|
| -32700 | Parse error | Check JSON syntax |
| -32600 | Invalid Request | Validate message structure |
| -32601 | Method not found | Check method name |
| -32602 | Invalid params | Verify parameter types |
| -32603 | Internal error | Check server logs |
| -32000 | Server error | Restart MCP server |

### 7. Development Workflow

**Setup:**
```bash
# Start MCP server in development mode
npm run mcp:dev

# In another terminal, start Tauri app
npm run tauri dev

# Run E2E tests
npm run test:e2e
```

**Hot Reload:**
- MCP server: `nodemon` or `cargo watch`
- Tauri app: Built-in hot reload
- Tests: `jest --watch`

### 8. Useful Commands

```bash
# Clean slate restart
./scripts/clean-mcp.sh

# Validate MCP messages
cat message.json | jq . && echo "Valid JSON"

# Test socket with timeout
timeout 5 nc -U /tmp/tauri-mcp-e2e.sock < test-message.json

# Monitor all socket activity
sudo tcpdump -i lo -A | grep tauri-mcp
```

## Environment Variables

```bash
# MCP Configuration
export MCP_SOCKET_PATH=/tmp/tauri-mcp-e2e.sock
export MCP_TIMEOUT=5000
export MCP_DEBUG=0

# Tauri Configuration
export TAURI_DEBUG=0
export WEBKIT_DISABLE_COMPOSITING_MODE=1
```

## Quick Recovery Steps

1. **Kill all related processes**
   ```bash
   pkill -f tauri-mcp
   pkill -f "tauri dev"
   ```

2. **Clean socket**
   ```bash
   rm -f /tmp/tauri-mcp-e2e.sock
   ```

3. **Restart in correct order**
   ```bash
   npm run mcp:start
   sleep 2
   npm run tauri dev
   ```

4. **Verify connection**
   ```bash
   echo '{"jsonrpc":"2.0","id":"1","method":"ping"}' | nc -U /tmp/tauri-mcp-e2e.sock
   ```

## Additional Resources

- MCP Protocol: https://modelcontextprotocol.io/
- Tauri IPC: https://v2.tauri.app/develop/calling-rust/
- Unix Sockets: `man 7 unix`
