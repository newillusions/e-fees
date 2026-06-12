# MCP Architecture in E-Fees
**Model Context Protocol Integration for AI-Driven E2E Testing**

## 🎯 Purpose

This document explains the custom MCP (Model Context Protocol) integration in e-fees, which enables Claude to directly interact with the running Tauri application for end-to-end testing, debugging, and development assistance.

---

## 📖 What is MCP?

**Model Context Protocol (MCP)** is a standard protocol that allows AI assistants to interact with external tools and systems through a well-defined interface.

### Why We Use MCP in E-Fees

**Traditional Testing Limitations**:
- ❌ Manual testing is slow and error-prone
- ❌ Standard E2E frameworks can't access internal app state
- ❌ Debugging requires recreating issues manually
- ❌ Test data cleanup is often incomplete

**MCP Solution**:
- ✅ Claude can invoke Tauri commands directly
- ✅ Real-time access to application state
- ✅ Automated test data creation and cleanup
- ✅ Intelligent debugging with full context
- ✅ Natural language test descriptions

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Claude Code Session                      │
│  (E2E Test Suite, Debugging, Development Assistance)         │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ MCP Protocol
                         │ (JSON-RPC over Unix Socket)
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              MCP Server (TypeScript)                         │
│  Location: tauri-plugin-mcp/mcp-server-ts/                  │
│  - Listens on: /tmp/tauri-mcp-e2e.sock                     │
│  - Tool definitions & routing                               │
│  - Request/response handling                                │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ IPC Communication
                         │ (Tauri Commands)
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Tauri Application (Rust)                        │
│  Location: src-tauri/                                        │
│  - Business logic                                           │
│  - Database operations (SurrealDB)                          │
│  - State management                                         │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔌 Socket-Based Communication

### Unix Domain Socket

**Socket Path**: `/tmp/tauri-mcp-e2e.sock`

**Why Unix Socket?**
- ✅ Fast IPC (no network overhead)
- ✅ Secure (local filesystem only)
- ✅ Simple lifecycle management
- ✅ No port conflicts

### Connection Lifecycle

```typescript
// 1. Tauri app starts
// 2. MCP plugin initializes
// 3. Server creates socket at /tmp/tauri-mcp-e2e.sock
// 4. Server listens for connections

// Claude Code connects:
const socket = net.connect('/tmp/tauri-mcp-e2e.sock')

// Exchange JSON-RPC messages:
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_contact",
    "arguments": { "name": "DELETE ME - Test", "email": "test@example.com" }
  }
}

// Response:
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { "id": "contact:abc123", "name": "DELETE ME - Test", ... }
}

// 5. Connection stays open for duration of test
// 6. Socket closes when Tauri app stops
```

---

## 🛠️ MCP Tool Definitions

### Tool Structure

Each MCP tool corresponds to a Tauri command or group of commands:

```typescript
// Location: tauri-plugin-mcp/mcp-server-ts/src/tools.ts

interface MCPTool {
  name: string              // Tool identifier
  description: string       // What it does
  inputSchema: JSONSchema   // Parameter validation
}

// Example: Contact creation tool
{
  name: "create_contact",
  description: "Create a new contact in the e-fees database",
  inputSchema: {
    type: "object",
    properties: {
      name: { type: "string" },
      email: { type: "string" },
      phone: { type: "string", optional: true },
      company_id: { type: "string", optional: true }
    },
    required: ["name", "email"]
  }
}
```

### Available Tool Categories

| Category | Purpose | Example Tools |
|----------|---------|---------------|
| **Contacts** | Contact CRUD operations | `create_contact`, `get_contact`, `delete_contact` |
| **Companies** | Company management | `create_company`, `list_companies`, `update_company` |
| **Invoices** | Invoice operations | `create_invoice`, `get_invoice_status` |
| **Projects** | Project tracking | `create_project`, `assign_project_contact` |
| **Testing** | Test utilities | `cleanup_test_data`, `verify_database_clean` |
| **Debug** | Debugging helpers | `get_app_state`, `dump_database_stats` |

### Tool Implementation Pattern

```typescript
// 1. Tool definition in MCP server
export const tools: MCPTool[] = [
  {
    name: "create_contact",
    description: "Create a new contact",
    inputSchema: { ... }
  }
]

// 2. Handler routes to Tauri command
async function handleToolCall(toolName: string, args: any) {
  switch (toolName) {
    case "create_contact":
      return await invokeTauriCommand("create_contact", args)
    // ... other cases
  }
}

// 3. Tauri command executes business logic
#[tauri::command]
pub async fn create_contact(
    contact: Contact,
    db: State<'_, Database>
) -> Result<Contact, String> {
    // Database operation
    let result = db.create_contact(contact).await?;
    Ok(result)
}
```

---

## 🧪 E2E Testing Integration

### Test Flow

```typescript
// File: e2e-mcp/src/tests/contacts.test.ts

describe('Contact Management', () => {
  
  // 1. Claude invokes MCP tool via test framework
  test('create and retrieve contact', async () => {
    
    // Create via MCP
    const contact = await mcp.invoke('create_contact', {
      name: `DELETE ME - Test Contact ${Date.now()}`,
      email: `delete-me-${Date.now()}@example.com`
    })
    
    expect(contact.id).toBeDefined()
    
    // Retrieve via MCP
    const retrieved = await mcp.invoke('get_contact', {
      id: contact.id
    })
    
    expect(retrieved.name).toBe(contact.name)
    
    // Cleanup via MCP
    await mcp.invoke('delete_contact', { id: contact.id })
  })
  
  // 2. Test cleanup verification
  afterAll(async () => {
    const remaining = await mcp.invoke('verify_database_clean')
    expect(remaining).toHaveLength(0)
  })
})
```

### Test Data Management

**"DELETE ME" Pattern**:

```typescript
// ✅ All test data MUST include this prefix
const testData = {
  name: `DELETE ME - ${entityType} ${Date.now()}`,
  email: `delete-me-${Date.now()}@example.com`
}

// Why?
// 1. Easy identification in database queries
// 2. Automated cleanup via pattern matching
// 3. Prevents test data leaking into production
// 4. Clear audit trail for debugging
```

**Cleanup Tool**:

```typescript
// MCP Tool: cleanup_test_data
// Finds and removes all entities with "DELETE ME" prefix

await mcp.invoke('cleanup_test_data', {
  dry_run: false  // Set true to preview without deleting
})

// Returns:
{
  contacts_deleted: 5,
  companies_deleted: 2,
  invoices_deleted: 3,
  total: 10
}
```

---

## 🔧 Tauri Plugin Integration

### Plugin Structure

```
tauri-plugin-mcp/
├── src/                      # Rust plugin code
│   ├── lib.rs               # Plugin initialization
│   ├── commands.rs          # MCP-specific commands
│   └── models.rs            # Data models
├── mcp-server-ts/           # TypeScript MCP server
│   ├── src/
│   │   ├── server.ts       # Main server logic
│   │   ├── tools.ts        # Tool definitions
│   │   └── ipc.ts          # Tauri IPC bridge
│   ├── package.json
│   └── tsconfig.json
├── build.rs                 # Build script
└── Cargo.toml
```

### Plugin Initialization

```rust
// File: tauri-plugin-mcp/src/lib.rs

use tauri::{plugin::Plugin, Runtime, AppHandle};

pub struct McpPlugin<R: Runtime> {
    socket_path: String,
    server_handle: Option<ServerHandle>,
}

impl<R: Runtime> Plugin<R> for McpPlugin<R> {
    fn initialize(&mut self, app: &AppHandle<R>) -> tauri::plugin::Result<()> {
        // 1. Start MCP server process
        let server = start_mcp_server(&self.socket_path)?;
        self.server_handle = Some(server);
        
        // 2. Register cleanup on app exit
        app.on_exit(|_| {
            cleanup_socket(&self.socket_path);
        });
        
        Ok(())
    }
}
```

### Server Startup

```typescript
// File: tauri-plugin-mcp/mcp-server-ts/src/server.ts

import * as net from 'net'
import * as fs from 'fs'

const SOCKET_PATH = '/tmp/tauri-mcp-e2e.sock'

export function startServer() {
  // Remove stale socket
  if (fs.existsSync(SOCKET_PATH)) {
    fs.unlinkSync(SOCKET_PATH)
  }
  
  // Create Unix socket server
  const server = net.createServer((socket) => {
    console.log('MCP client connected')
    
    socket.on('data', async (data) => {
      const request = JSON.parse(data.toString())
      const response = await handleRequest(request)
      socket.write(JSON.stringify(response))
    })
  })
  
  server.listen(SOCKET_PATH)
  console.log(`MCP server listening on ${SOCKET_PATH}`)
}
```

---

## 🔍 Debugging & Monitoring

### Connection Troubleshooting

```bash
# Check if socket exists
ls -la /tmp/tauri-mcp-e2e.sock

# Check if MCP server is running
ps aux | grep mcp-server

# Test socket connectivity
nc -U /tmp/tauri-mcp-e2e.sock

# View MCP server logs
tail -f tauri-plugin-mcp/mcp-server-ts/logs/server.log
```

### Common Issues

| Problem | Cause | Solution |
|---------|-------|----------|
| Socket not found | Tauri app not running | Start: `npm run tauri:dev` |
| Connection refused | Stale socket file | Remove: `rm /tmp/tauri-mcp-e2e.sock` |
| Tool not found | MCP server outdated | Rebuild: `cd tauri-plugin-mcp/mcp-server-ts && npm run build` |
| Timeout errors | Server crashed | Check logs, restart Tauri app |

### Debug Mode

```typescript
// Enable verbose MCP logging
// File: tauri-plugin-mcp/mcp-server-ts/src/config.ts

export const DEBUG_MODE = process.env.MCP_DEBUG === 'true'

if (DEBUG_MODE) {
  console.log('[MCP] Request:', JSON.stringify(request))
  console.log('[MCP] Response:', JSON.stringify(response))
  console.log('[MCP] Timing:', executionTime, 'ms')
}
```

**Run with debug mode**:
```bash
MCP_DEBUG=true npm run tauri:dev
```

---

## 📊 Performance Considerations

### Request Latency

**Target**: < 100ms per tool invocation

**Typical Latencies**:
- Simple reads (get_contact): 5-15ms
- Writes (create_contact): 10-30ms
- Bulk operations: 50-100ms
- Complex queries: 100-500ms

### Optimization Strategies

```typescript
// 1. Batch operations
// ❌ BAD - 100 individual calls
for (const contact of contacts) {
  await mcp.invoke('create_contact', contact)
}

// ✅ GOOD - Single batch call
await mcp.invoke('batch_create_contacts', { contacts })

// 2. Connection pooling
// Keep socket connection open across test suite
// Don't reconnect for every tool call

// 3. Async operations
// Use Promise.all for parallel operations
await Promise.all([
  mcp.invoke('create_contact', contact1),
  mcp.invoke('create_company', company1),
  mcp.invoke('create_project', project1)
])
```

---

## 🔐 Security Considerations

### Socket Permissions

```bash
# Socket should be readable/writable only by owner
chmod 600 /tmp/tauri-mcp-e2e.sock

# Verify permissions
ls -la /tmp/tauri-mcp-e2e.sock
# Expected: srw------- 1 user user 0 date /tmp/tauri-mcp-e2e.sock
```

### Input Validation

```typescript
// All tool inputs are validated against JSON schema
// Example: email validation
{
  type: "string",
  format: "email",
  pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
}

// Invalid inputs are rejected before reaching Tauri
const result = await mcp.invoke('create_contact', {
  email: "invalid-email"  // ❌ Rejected by schema validation
})
```

### Test Data Isolation

```typescript
// Test data must be clearly marked
// Prevents accidental deletion of real data
const isTestData = (name: string) => name.includes('DELETE ME')

// Cleanup only removes test data
await db.delete('contact', { 
  where: { name: { contains: 'DELETE ME' } }
})
```

---

## 🚀 Advanced Features

### State Inspection

```typescript
// MCP tool: get_app_state
// Returns current application state for debugging

const state = await mcp.invoke('get_app_state')

// Response:
{
  database: {
    contacts: 42,
    companies: 15,
    invoices: 128,
    projects: 23
  },
  memory_usage: "45.2 MB",
  uptime: "2h 15m 30s",
  active_sessions: 1
}
```

### Database Snapshots

```typescript
// Create snapshot before risky operations
await mcp.invoke('create_database_snapshot', {
  name: 'before-migration'
})

// Run operation
await runMigration()

// Restore if needed
await mcp.invoke('restore_database_snapshot', {
  name: 'before-migration'
})
```

### Live Debugging

```typescript
// Set breakpoint-like watches via MCP
await mcp.invoke('watch_entity', {
  entity_type: 'contact',
  entity_id: 'contact:abc123',
  notify_on: ['update', 'delete']
})

// Receive notifications when entity changes
// Useful for debugging race conditions
```

---

## 🔄 Future Enhancements

### Planned Features

1. **Recording & Playback**
   - Record MCP interactions during manual testing
   - Replay as automated tests
   - Useful for regression testing

2. **Visual Test Reports**
   - Screenshot capture during E2E tests
   - HTML reports with step-by-step breakdowns
   - Integration with CI/CD pipeline

3. **Remote MCP Access**
   - TCP socket option for remote debugging
   - Secure authentication
   - Multi-client support

4. **AI-Driven Test Generation**
   - Claude analyzes app structure
   - Generates comprehensive test suites
   - Suggests edge cases and scenarios

5. **Performance Profiling**
   - Track tool invocation times
   - Identify slow operations
   - Automated performance regression detection

---

## 📝 Adding New MCP Tools

### Step-by-Step Guide

```typescript
// 1. Define tool in mcp-server-ts/src/tools.ts
export const tools: MCPTool[] = [
  {
    name: "new_tool_name",
    description: "What this tool does",
    inputSchema: {
      type: "object",
      properties: {
        param1: { type: "string" },
        param2: { type: "number" }
      },
      required: ["param1"]
    }
  }
]

// 2. Add handler in mcp-server-ts/src/handlers.ts
async function handleToolCall(name: string, args: any) {
  switch (name) {
    case "new_tool_name":
      return await invokeTauriCommand("new_tauri_command", args)
  }
}

// 3. Implement Tauri command in src-tauri/src/commands/
#[tauri::command]
pub async fn new_tauri_command(
    param1: String,
    param2: Option<i32>
) -> Result<ResponseType, String> {
    // Implementation
}

// 4. Register in src-tauri/src/main.rs
.invoke_handler(tauri::generate_handler![
    new_tauri_command,
    // ... other commands
])

// 5. Add TypeScript types in src/lib/types/
export interface NewToolParams {
  param1: string
  param2?: number
}

// 6. Write integration test in e2e-mcp/src/tests/
test('new tool works', async () => {
  const result = await mcp.invoke('new_tool_name', {
    param1: 'test'
  })
  expect(result).toBeDefined()
})

// 7. Update documentation
// - Add to tool list in this file
// - Update .claude/prompts/mcp-troubleshooting.md if complex
```

---

## 🔗 Related Documentation

- **MCP Troubleshooting**: `.claude/prompts/mcp-troubleshooting.md`
- **Testing Strategy**: `.claude/context/testing-strategy.md`
- **Tauri Development**: `.claude/prompts/tauri-development.md`
- **Sub-Agent for MCP**: `.claude/subagents/subagent-mcp-specialist.md`

---

## 📚 External Resources

- **MCP Specification**: https://modelcontextprotocol.io
- **Tauri IPC Documentation**: https://tauri.app/v2/guides/inter-process-communication/
- **SurrealDB Documentation**: https://surrealdb.com/docs
- **JSON-RPC 2.0 Spec**: https://www.jsonrpc.org/specification

---

## ✅ Best Practices Summary

### DO
- ✅ Use "DELETE ME" prefix in all test data
- ✅ Verify socket exists before running tests
- ✅ Keep socket connections alive during test suites
- ✅ Validate inputs with JSON schema
- ✅ Clean up test data after every test run
- ✅ Log all tool invocations in debug mode
- ✅ Use batch operations for bulk data

### DON'T
- ❌ Create test data without "DELETE ME" prefix
- ❌ Leave stale socket files
- ❌ Skip cleanup verification steps
- ❌ Hardcode socket paths (use config)
- ❌ Ignore MCP server errors
- ❌ Run tests against production database

---

## 🎯 Quick Reference

```bash
# Start development with MCP
npm run tauri:dev

# Run E2E tests via MCP
npm run test:e2e

# Verify test data cleanup
npm run test:e2e:verify-clean

# Debug MCP connection
MCP_DEBUG=true npm run tauri:dev

# Rebuild MCP server
cd tauri-plugin-mcp/mcp-server-ts && npm run build

# Remove stale socket
rm /tmp/tauri-mcp-e2e.sock
```

---

**Last Updated**: October 26, 2025  
**Version**: 1.0  
**Maintained By**: Martin & Claude Code  
**Socket Path**: `/tmp/tauri-mcp-e2e.sock`
