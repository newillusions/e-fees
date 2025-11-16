# MCP Server Configuration Guide

> **Purpose**: Comprehensive guide to MCP (Model Context Protocol) servers used in the e-fees project for testing, automation, and development workflows.
>
> **Last Updated**: November 6, 2025

---

## 📋 Table of Contents

- [Quick Reference](#quick-reference)
- [Available MCP Servers](#available-mcp-servers)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Adding New MCP Servers](#adding-new-mcp-servers)

---

## Quick Reference

### Active MCP Servers

| Server | Purpose | Status | Documentation |
|--------|---------|--------|---------------|
| **Tauri MCP** | E2E testing & app automation | Custom/Active | [Details](#1-tauri-mcp-server-custom) |
| **Desktop Commander** | File operations & process management | Active | [Details](#2-desktop-commander-mcp) |
| **Filesystem** | File system operations | Active | [Details](#3-filesystem-mcp) |
| **Playwright/Puppeteer** | Browser automation | Active | [Details](#4-browser-automation-mcp) |
| **SurrealDB** | Database operations | Configured | [Details](#5-surrealdb-mcp) |
| **Memory** | Knowledge graph | Active | [Details](#6-memory-mcp) |
| **Sequential Thinking** | Complex reasoning | Active | [Details](#7-sequential-thinking-mcp) |

### Quick Health Check

```bash
# Check if Tauri MCP socket exists
ls -la /tmp/tauri-mcp-e2e.sock

# Verify app is running (required for Tauri MCP)
ps aux | grep -i "e-fees" | grep -v grep

# Test MCP server connectivity
npm run test:e2e:mcp -- --grep "basic"
```

---

## Available MCP Servers

### 1. Tauri MCP Server (Custom)

**Purpose**: Custom MCP integration for E2E testing of the Tauri application

**Type**: Project-specific, built-in plugin
**Location**: `tauri-plugin-mcp/`
**Socket Path**: `/tmp/tauri-mcp-e2e.sock`

#### How It Works

1. Tauri app starts the MCP server on launch
2. Creates Unix socket at `/tmp/tauri-mcp-e2e.sock`
3. Test framework connects via socket
4. Executes commands via JSON-RPC protocol

#### Starting the Server

```bash
# Server starts automatically with Tauri app
npm run tauri:dev

# Verify socket created
ls -la /tmp/tauri-mcp-e2e.sock
# Should show: srwxr-xr-x  1 user  wheel  0 [date] /tmp/tauri-mcp-e2e.sock
```

#### Available Commands

The Tauri MCP server provides:
- Window management (resize, position, focus)
- DOM inspection and interaction
- JavaScript execution in app context
- Screenshot capture
- Local storage operations
- Mouse/keyboard simulation

#### Configuration

**In Tauri app** (`src-tauri/src/main.rs`):
```rust
// MCP plugin automatically registered
.plugin(tauri_plugin_mcp::init())
```

**In tests** (`e2e-mcp/tests/*.spec.ts`):
```typescript
import { MCPClient } from '../mcp-client'

const client = new MCPClient('/tmp/tauri-mcp-e2e.sock')
await client.connect()
```

#### Troubleshooting

**Socket not found**:
```bash
# Kill any stale processes
pkill -f tauri-mcp

# Remove old socket
rm -f /tmp/tauri-mcp-e2e.sock

# Restart app
npm run tauri:dev

# Wait 2-3 seconds, then verify
ls -la /tmp/tauri-mcp-e2e.sock
```

**Connection refused**:
- Ensure Tauri app is running
- Check for port conflicts
- Review logs: `RUST_LOG=debug npm run tauri:dev`

**See also**: [MCP Specialist](subagents/subagent-mcp-specialist.md) for detailed troubleshooting

---

### 2. Desktop Commander MCP

**Purpose**: Advanced file operations, process management, and system interactions

**Type**: External package
**Package**: `@desktopcommander/mcp-server`

#### Capabilities

- **File Operations**: Read, write, edit, search
- **Process Management**: Start, monitor, interact with processes
- **Directory Operations**: List, search, tree view
- **Code Search**: Content and pattern matching

#### Usage Examples

```typescript
// Read file with range
await mcp.read_file({
  path: '/path/to/file.ts',
  offset: 100,
  length: 50
})

// Start interactive process (Python REPL)
await mcp.start_process({
  command: 'python3 -i',
  timeout_ms: 30000
})

// Search code
await mcp.start_search({
  path: './src',
  pattern: 'createContact',
  searchType: 'content'
})
```

#### Configuration

Desktop Commander is configured via environment variables and settings:

```bash
# Check current config
mcp__desktop-commander__get_config
```

**Allowed directories** (already configured in settings.local.json):
- `/Volumes/base/dev` - Full development directory access

---

### 3. Filesystem MCP

**Purpose**: Standard file system operations

**Type**: Built-in Claude MCP
**Package**: `@modelcontextprotocol/server-filesystem`

#### Capabilities

- Read files (text and media)
- Write files
- Directory operations
- File search
- Metadata inspection

#### Usage Pattern

```typescript
// Read file
await mcp.read_text_file({ path: '/path/to/file.ts' })

// List directory
await mcp.list_directory({ path: '/path/to/dir' })

// Search files
await mcp.search_files({
  path: '/path/to/search',
  pattern: '*.ts'
})
```

#### Allowed Directories

Configured in settings.local.json:
```json
{
  "additionalDirectories": [
    "/Volumes/base/dev"
  ]
}
```

---

### 4. Browser Automation MCP

**Purpose**: Browser-based E2E testing and automation

**Servers Available**:
- **Playwright MCP** - `@playwright/mcp-server`
- **Puppeteer MCP** - `@puppeteer/mcp-server`
- **Browser MCP** - `@browser/mcp-server`

#### Capabilities

- Page navigation
- Element interaction (click, type, select)
- Screenshots and snapshots
- JavaScript evaluation
- Form filling
- Network monitoring

#### Usage Example

```typescript
// Navigate and take screenshot
await mcp.browser_navigate({ url: 'http://localhost:1421' })
await mcp.browser_snapshot()
await mcp.browser_click({ element: 'Submit button', ref: '[ref-id]' })
```

---

### 5. SurrealDB MCP

**Purpose**: Direct database operations via MCP

**Type**: External package
**Package**: `surrealdb-mcp-server`

#### Setup Required

**Option 1: Local Database** (Recommended for e-fees)
```bash
# Add to .env.local
SURREALDB_URL="file:./data/efees.db"
SURREALDB_NS="efees"
SURREALDB_DB="main"
SURREALDB_USER="root"
SURREALDB_PASS="root"
```

**Option 2: Remote Database** (If using external SurrealDB)
```bash
# Add to .env.local
SURREALDB_URL="ws://10.0.1.17:8000/rpc"
SURREALDB_NS="e-fees-dev"
SURREALDB_DB="test"
SURREALDB_USER="${DB_USER}"
SURREALDB_PASS="${DB_PASS}"
```

#### Configuration in Claude Desktop

**File**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "surrealdb-efees": {
      "command": "npx",
      "args": ["-y", "surrealdb-mcp-server@latest"],
      "env": {
        "SURREALDB_URL": "file:./data/efees.db",
        "SURREALDB_NS": "efees",
        "SURREALDB_DB": "main"
      }
    }
  }
}
```

#### Usage

```typescript
// Create record
await mcp.surrealdb_create({
  target: 'contacts',
  data: { name: 'John Doe', email: 'john@example.com' }
})

// Query
await mcp.surrealdb_query({
  query: 'SELECT * FROM contacts WHERE name CONTAINS $name',
  parameters: { name: 'John' }
})
```

#### Current Status

**Status**: Configured but not actively used
**Reason**: App uses direct SurrealDB connection via Rust backend
**Use Case**: Useful for debugging, ad-hoc queries, and test data inspection

---

### 6. Memory MCP

**Purpose**: Knowledge graph for storing and retrieving context across sessions

**Type**: Built-in Claude MCP
**Package**: `@modelcontextprotocol/server-memory`

#### Capabilities

- Create entities and relations
- Add observations
- Search nodes
- Read full knowledge graph

#### Usage Pattern

```typescript
// Create entities
await mcp.create_entities({
  entities: [{
    name: 'E-Fees Project',
    entityType: 'project',
    observations: ['Tauri-based desktop app', 'Uses SurrealDB']
  }]
})

// Create relations
await mcp.create_relations({
  relations: [{
    from: 'E-Fees Project',
    to: 'Tauri',
    relationType: 'built_with'
  }]
})
```

---

### 7. Sequential Thinking MCP

**Purpose**: Complex reasoning and problem-solving through structured thought chains

**Type**: Built-in Claude MCP
**Package**: `@modelcontextprotocol/server-sequential-thinking`

#### When to Use

- Multi-step problem analysis
- Planning complex features
- Debugging intricate issues
- Decision-making with trade-offs

---

## Configuration

### Global MCP Configuration

**Location**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Current servers** (based on active usage):
```json
{
  "mcpServers": {
    "desktop-commander": {
      "command": "desktop-commander",
      "args": ["serve"]
    },
    "playwright": {
      "command": "npx",
      "args": ["-y", "@playwright/mcp-server"]
    }
  }
}
```

### Project-Specific Configuration

**Location**: `.claude/settings.local.json`

See [settings.local.json](settings.local.json) for:
- Bash command permissions
- MCP tool permissions
- Additional directories
- WebFetch domains

---

## Troubleshooting

### General MCP Issues

**MCP server not responding**:
```bash
# Check Claude Desktop MCP servers
# Settings → Developer → MCP Servers

# Restart Claude Desktop
killall "Claude" && open -a "Claude"
```

**Tool not found**:
```bash
# Verify MCP server is configured
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Check package installation
npx -y [package-name] --version
```

### Project-Specific Issues

**Tauri MCP socket issues**: See [MCP Specialist](subagents/subagent-mcp-specialist.md)
**Test failures**: See [Testing Specialist](subagents/subagent-testing-specialist.md)
**Database MCP issues**: See [Database Specialist](subagents/subagent-database-specialist.md)

---

## Adding New MCP Servers

### Steps to Add a New MCP Server

1. **Identify the server**
   ```bash
   # Search npm for MCP servers
   npm search mcp-server
   ```

2. **Test locally**
   ```bash
   npx -y [mcp-package-name]
   ```

3. **Add to Claude Desktop config**
   ```json
   {
     "mcpServers": {
       "server-name": {
         "command": "npx",
         "args": ["-y", "package-name"],
         "env": {
           "KEY": "value"
         }
       }
     }
   }
   ```

4. **Update project permissions**
   Add to `.claude/settings.local.json`:
   ```json
   {
     "permissions": {
       "allow": [
         "mcp__server-name__*"
       ]
     }
   }
   ```

5. **Document usage**
   Add section to this file with:
   - Purpose
   - Configuration
   - Usage examples
   - Troubleshooting

### Recommended Additional Servers

**Git MCP** (for extensive git operations):
```json
{
  "git": {
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-git"]
  }
}
```

**Context7 MCP** (for library documentation):
```json
{
  "context7": {
    "command": "npx",
    "args": ["-y", "@context7/mcp-server"]
  }
}
```

---

## Best Practices

### 1. Security
- ❌ Never commit credentials to settings files
- ✅ Use environment variables for sensitive data
- ✅ Keep `.env.local` in `.gitignore`

### 2. Performance
- ✅ Use Tauri MCP for app-specific operations (faster)
- ✅ Use Desktop Commander for file operations (optimized)
- ⚠️ Avoid excessive MCP calls in tight loops

### 3. Testing
- ✅ Always verify MCP server availability before tests
- ✅ Clean up test data using "DELETE ME" prefix
- ✅ Run `npm run test:e2e:verify-clean` after tests

### 4. Maintenance
- 📅 Review MCP server versions quarterly
- 📅 Clean up unused permissions monthly
- 📅 Update documentation when adding new servers

---

## Quick Commands

```bash
# Start Tauri MCP
npm run tauri:dev

# Verify Tauri MCP socket
ls -la /tmp/tauri-mcp-e2e.sock

# Run MCP E2E tests
npm run test:e2e:mcp

# Clean test data
npm run test:e2e:cleanup

# Check Claude Desktop MCP config
cat ~/Library/Application\ Support/Claude/claude_desktop_config.json

# Restart Claude Desktop
killall "Claude" && open -a "Claude"
```

---

## Related Documentation

- [MCP Architecture](context/mcp-architecture.md) - Deep dive into MCP integration
- [MCP Specialist](subagents/subagent-mcp-specialist.md) - Troubleshooting expert
- [Testing Strategy](context/testing-strategy.md) - E2E testing approach
- [MCP Troubleshooting](prompts/mcp-troubleshooting.md) - Detailed debugging guide

---

**Need Help?** Consult the [MCP Specialist](subagents/subagent-mcp-specialist.md) for socket issues, protocol problems, or test framework debugging.
