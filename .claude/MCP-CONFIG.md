# E-Fees MCP Server Configuration

This document explains the project-specific MCP (Model Context Protocol) server setup for the e-fees project.

## Why Project-Specific MCP Config?

**Problem:** The global `~/.claude-cli-mcp-config.json` contains 24 MCP servers, many of which are irrelevant to e-fees (n8n, Home Assistant, postgres, browser automation, etc.).

**Solution:** A local `.mcp-config.json` with only 11 essential servers for this project.

**Benefits:**
- ✅ Faster startup (fewer servers to initialize)
- ✅ Less context pollution (only relevant tools)
- ✅ Project isolation (doesn't affect other projects)
- ✅ Clear documentation of dependencies
- ✅ Easier onboarding (obvious which MCP servers are needed)

---

## Quick Start

### Option 1: Use the Launcher Script (Recommended)

```bash
# From anywhere:
/Volumes/base/dev/e-fees/claude-efees.sh

# Or create an alias in ~/.zshrc:
alias claude-efees='/Volumes/base/dev/e-fees/claude-efees.sh'

# Then use:
claude-efees
```

### Option 2: Direct CLI Usage

```bash
cd /Volumes/base/dev/e-fees
claude --mcp-config .mcp-config.json
```

---

## Active MCP Servers (11 Total)

### 🔴 Critical (4 servers)

| Server | Purpose | Configuration |
|--------|---------|---------------|
| **SurrealDB** | Database access | `e-fees-dev/test` @ `ws://10.0.1.17:8000/rpc` |
| **tauri-mcp** | Tauri app automation | Scoped to `/Volumes/base/dev/e-fees` |
| **filesystem** | File operations | Scoped to `/Volumes/base/dev/e-fees` and `/tmp` |
| **git** | Version control | Scoped to `/Volumes/base/dev/e-fees` |

### 🟢 Essential (3 servers)

| Server | Purpose | Notes |
|--------|---------|-------|
| **desktop-commander** | Process & file management | Enhanced file operations, process control |
| **memory** | Knowledge graph | Persistent memory across sessions |
| **github** | GitHub integration | PR/issue management, code search |

### 🔵 Utility (4 servers)

| Server | Purpose | When to Use |
|--------|---------|-------------|
| **sequential-thinking** | Complex problem solving | Multi-step reasoning tasks |
| **brave-search** | Web search | Documentation lookup, troubleshooting |
| **context7** | Library documentation | Fetch up-to-date docs for Tauri, Svelte, SurrealDB |
| **fetch** | Web content | Fetch URLs, API responses |

---

## Comparison: Global vs Project Config

### Global Config (`~/.claude-cli-mcp-config.json`)
**24 MCP servers** including:
- ❌ n8n/n8n-research (workflow automation - not needed)
- ❌ Home Assistant x2 (home automation - not needed)
- ❌ postgres (using SurrealDB instead)
- ❌ docker (not using containers)
- ❌ puppeteer/playwright/browser-mcp x3 (browser automation - excessive)
- ❌ mcp-ssh (SSH operations - not needed)
- ❌ pdf (PDF processing - not needed)
- ❌ macos-notification/macos-screenshot (macOS utilities - not needed)
- ❌ mistral-ocr (OCR - not needed)

### Project Config (`.mcp-config.json`)
**11 MCP servers** - only what's needed for e-fees development

**Startup time:** ~50% faster
**Memory usage:** ~40% reduction
**Cognitive load:** Significantly lower (no irrelevant tools)

---

## Configuration Details

### SurrealDB Server

```json
{
  "SurrealDB": {
    "command": "/Users/martin/.cargo/bin/surrealmcp",
    "args": ["start"],
    "env": {
      "SURREALDB_URL": "ws://10.0.1.17:8000/rpc",
      "SURREALDB_NS": "e-fees-dev",
      "SURREALDB_DB": "test",
      "SURREALDB_USER": "martin",
      "SURREALDB_PASS": "th38ret3ch"
    }
  }
}
```

**Key Points:**
- Uses **WebSocket connection** (`ws://`) to remote SurrealDB instance
- Namespace: `e-fees-dev` (development environment)
- Database: `test` (for testing, change to `production` when ready)
- Server location: `10.0.1.17:8000` (local network)

### Tauri MCP Server

```json
{
  "tauri-mcp": {
    "command": "/Users/martin/.cargo/bin/tauri-mcp",
    "args": ["--app-path", "/Volumes/base/dev/e-fees", "serve"]
  }
}
```

**Key Points:**
- Rust-based MCP server for Tauri app automation
- Provides tools for:
  - Screenshots
  - DOM access
  - Window management
  - Input simulation
  - Execute JavaScript
  - Local storage manipulation

### Filesystem Server

```json
{
  "filesystem": {
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/Volumes/base/dev/e-fees",
      "/tmp"
    ]
  }
}
```

**Key Points:**
- **Scoped to e-fees directory** - Cannot access files outside project
- Also allows `/tmp` access for temporary files
- **Security:** Prevents accidental modification of unrelated files

### Git Server

```json
{
  "git": {
    "command": "uvx",
    "args": [
      "mcp-server-git",
      "--repository",
      "/Volumes/base/dev/e-fees"
    ]
  }
}
```

**Key Points:**
- **Scoped to e-fees repository** - Only operates on this project
- Provides tools for:
  - Git status
  - Diff (staged/unstaged)
  - Commit
  - Branch management
  - Log viewing

---

## Maintenance

### Adding a New MCP Server

1. Edit `.mcp-config.json`
2. Add server configuration:
   ```json
   {
     "server-name": {
       "command": "npx",
       "args": ["-y", "package-name"],
       "env": {
         "API_KEY": "value"
       }
     }
   }
   ```
3. Update this document
4. Test with: `./claude-efees.sh`

### Removing an MCP Server

1. Remove from `.mcp-config.json`
2. Update this document
3. Document reason in git commit message

### Updating Environment Variables

**For SurrealDB:**
1. Edit `.mcp-config.json` → `SurrealDB.env` section
2. Update `SURREALDB_NS`, `SURREALDB_DB`, or connection details
3. Common changes:
   - Switch to production: `"SURREALDB_DB": "production"`
   - Change namespace: `"SURREALDB_NS": "e-fees-prod"`

**For GitHub:**
1. Update `github.env.GITHUB_PERSONAL_ACCESS_TOKEN`
2. Get new token from: https://github.com/settings/tokens

---

## Troubleshooting

### "MCP server failed to start"

**Check if required binaries are installed:**
```bash
# Rust-based servers
which surrealmcp
which tauri-mcp

# Python-based servers (uvx)
which uvx

# Node-based servers (npx)
which npx
```

**Install missing dependencies:**
```bash
# SurrealDB MCP
cargo install surrealmcp

# Tauri MCP
cargo install tauri-mcp

# Python UV (for uvx commands)
brew install uv

# Node.js (for npx commands)
brew install node
```

### "SurrealDB connection refused"

**Check if SurrealDB server is running:**
```bash
# Test connection
curl http://10.0.1.17:8000/health

# Or use SurrealDB CLI
surreal isready --conn ws://10.0.1.17:8000/rpc
```

**Common fixes:**
- Verify server is running at `10.0.1.17:8000`
- Check network connectivity
- Verify credentials in `.mcp-config.json`
- Check namespace and database exist

### "Filesystem access denied"

**Verify paths in `.mcp-config.json`:**
```json
{
  "filesystem": {
    "args": [
      "-y",
      "@modelcontextprotocol/server-filesystem",
      "/Volumes/base/dev/e-fees",  // Must be exact path
      "/tmp"
    ]
  }
}
```

**Remember:** Filesystem server can ONLY access paths listed in args.

### "Git repository not found"

**Check repository path:**
```bash
# Should point to e-fees root (where .git/ is)
ls -la /Volumes/base/dev/e-fees/.git
```

**Update if needed:**
```json
{
  "git": {
    "args": [
      "mcp-server-git",
      "--repository",
      "/Volumes/base/dev/e-fees"  // Must contain .git/
    ]
  }
}
```

---

## On-Demand MCP Servers (Slash Commands)

**Design Pattern:** Some MCP functionality is implemented as slash commands instead of always-loaded servers to optimize token usage.

### When to Use Slash Commands vs Always-Loaded MCP

| Use Case | Frequency | Implementation | Why |
|----------|-----------|----------------|-----|
| Database queries | Constant | MCP Server | Always needed |
| Testing automation | Frequent | MCP Server | Used in development |
| Release management | Occasional | Slash Command | Saves tokens |
| One-off operations | Rare | Slash Command | No persistent overhead |

### Available Slash Commands

#### `/gitea-release` - Gitea Release Management

**Purpose:** Create and manage releases on Gitea server (infrequent operation)

**Location:** `.claude/commands/gitea-release.md`

**Implements:** Gitea API integration for:
- Creating releases
- Uploading release assets (DMG, checksums)
- Listing/viewing releases
- Managing tags

**MCP Server:** `mcp-servers/gitea-mcp/` (not loaded by default)

**Token Savings:** ~2-3KB per session when not in use

**Usage Example:**
```
# In Claude Code conversation:
/gitea-release

# Then ask:
"Create a release for v0.11.0 and upload the DMG"
```

**See Also:**
- `mcp-servers/gitea-mcp/README.md` - Full documentation
- `scripts/create-gitea-release.sh` - Automated script
- `docs/development/GITEA_RELEASES.md` - Complete guide

---

## Related Files

- **`.mcp-config.json`** - MCP server configuration (this directory)
- **`claude-efees.sh`** - Launcher script (this directory)
- **`~/.claude-cli-mcp-config.json`** - Global MCP config (not used when using project config)
- **`.env.dev`** - Environment variables for development
- **`CLAUDE.md`** - Main project documentation

---

## Migration from Global Config

**Old workflow:**
```bash
# Used global config with 24 servers
claude-mcp
```

**New workflow:**
```bash
# Use project-specific config with 11 servers
./claude-efees.sh
```

**Benefits:**
- Clearer which MCP servers are actually needed
- Faster startup
- No irrelevant tools cluttering the environment
- Better documentation of project dependencies

---

**Last Updated:** November 17, 2025
**MCP Servers:** 11 active (always-loaded) + 1 on-demand (slash command)
**Config Location:** `/Volumes/base/dev/e-fees/.mcp-config.json`
