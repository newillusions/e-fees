# E-Fees Project Commands Reference

Complete guide to all available slash commands for this project.

## What Are Slash Commands?

Slash commands are custom shortcuts that load specific context and instructions into Claude Code without consuming tokens until invoked. They're defined in `.claude/commands/` and triggered by typing `/command-name` in the chat.

**Benefits:**
- ✅ Zero token cost until used
- ✅ Consistent, repeatable workflows
- ✅ Self-documenting project operations
- ✅ On-demand context loading

---

## How to Use Slash Commands

### Basic Syntax
```
/command-name
```

### With Arguments (if supported)
```
/command-name arg1 arg2
```

### Follow-up Interaction
After invoking a command, Claude will have the context loaded and you can ask questions or request actions related to that command's domain.

**Example Flow:**
```
You: /gitea-release
Claude: [Loads Gitea release management context]
You: "Create a release for v0.11.0 and upload the DMG"
Claude: [Executes release creation using loaded context]
```

---

## Available Commands

### `/commands` - This Guide
**Purpose:** Display comprehensive slash command reference

**When to Use:**
- Learning available commands
- Understanding command syntax
- Finding examples

**Example:**
```
/commands
```

**Follow-up:**
```
"Show me how to use the gitea-release command"
"What commands are available for testing?"
```

---

### `/gitea-release` - Gitea Release Management
**Purpose:** Create and manage releases on Gitea server

**When to Use:**
- After building a new release version
- Uploading release binaries (DMG, MSI, etc.)
- Managing existing releases
- Creating formal releases from git tags

**Syntax:**
```
/gitea-release
```

**Follow-up Examples:**
```
"Create a release for v0.11.0"
"Upload the DMG file for v0.11.0"
"List all releases"
"Delete release v0.10.0-beta"
"Show me the details for v0.10.0"
```

**What It Provides:**
- Access to Gitea MCP server (mcp-servers/gitea-mcp/)
- Credentials loaded from .env.local
- Tools: create_release, upload_release_asset, list_releases, get_release, delete_release, list_tags

**Alternative:**
```bash
# Direct script usage (no slash command needed):
./scripts/create-gitea-release.sh 0.11.0
```

**Files Referenced:**
- `mcp-servers/gitea-mcp/index.js` - MCP server implementation
- `mcp-servers/gitea-mcp/README.md` - Full documentation
- `scripts/create-gitea-release.sh` - Automated script
- `.env.local` - Credentials (GITEA_TOKEN, GITEA_SERVER, etc.)
- `docs/development/GITEA_RELEASES.md` - Complete guide

**Common Workflow:**
```bash
# 1. Set version
npm run version:set 0.11.0

# 2. Build release
npm run tauri:build

# 3. Create git tag
git tag -a v0.11.0 -m "Release v0.11.0"
git push origin v0.11.0

# 4. Use slash command
/gitea-release
"Create release for v0.11.0 and upload all artifacts from releases/v0.11.0/"
```

---

## Future Commands (Not Yet Implemented)

These are suggestions for future slash commands based on common workflows:

### `/test` - Testing Operations
**Potential Use:**
- Run E2E tests
- Clean test data
- Generate test reports
- Debug test failures

**Would Provide:**
- Test data cleanup commands
- MCP socket troubleshooting
- Test execution patterns

### `/db` - Database Operations
**Potential Use:**
- Schema exploration
- Query optimization
- Database migrations
- Backup/restore operations

**Would Provide:**
- SurrealDB connection details
- Query patterns
- Thing object handling
- Schema documentation

### `/build` - Build & Deploy
**Potential Use:**
- Production builds
- Multi-platform compilation
- Version management
- Distribution preparation

**Would Provide:**
- Build commands for all platforms
- Code signing procedures
- DMG creation
- Installer generation

### `/mcp` - MCP Troubleshooting
**Potential Use:**
- Socket debugging
- MCP server management
- Protocol inspection
- Connection testing

**Would Provide:**
- Socket cleanup procedures
- Server restart commands
- Log inspection
- Connection diagnostics

### `/specialist` - Sub-Agent Routing
**Potential Use:**
- Find the right specialist for a task
- Understand agent capabilities
- Coordinate multi-agent work

**Would Provide:**
- Agent routing guide
- Specialist summaries
- Task delegation patterns

---

## Creating Your Own Commands

### File Structure
```
.claude/commands/
  ├── command-name.md     # Your command file
  └── README.md           # Optional: commands documentation
```

### Template
```markdown
# Command Name

Brief description of what this command does.

## What You Should Do

When this command is invoked, help the user with [specific task area].

### Available Operations

1. **Operation 1**
2. **Operation 2**
3. **Operation 3**

### How to Execute

[Detailed instructions on what steps to take]

### When to Use This Command

- **Scenario 1** - Description
- **Scenario 2** - Description

### Files to Reference

- **File 1**: `path/to/file` - Purpose
- **File 2**: `path/to/file` - Purpose

### Example Usage

\`\`\`bash
# Example commands
command here
\`\`\`

### Security Notes

[Any security considerations]
```

### Best Practices

1. **Keep Commands Focused**
   - One command per domain/workflow
   - Don't try to do too much in one command

2. **Token Optimization**
   - Use commands for infrequent operations
   - Avoid loading heavy context by default
   - Prefer commands over always-loaded MCP servers for rare tasks

3. **Clear Instructions**
   - Explain what the command provides
   - Show concrete examples
   - List relevant files/scripts

4. **Self-Contained**
   - Include all necessary context
   - Reference external docs where appropriate
   - Provide fallback options (scripts, manual steps)

---

## Command vs MCP Server vs Script

### When to Use Each:

| Type | Frequency | Complexity | Token Cost | Best For |
|------|-----------|------------|------------|----------|
| **Slash Command** | Occasional | Medium-High | Zero until used | Release management, admin tasks |
| **MCP Server** | Frequent | High | ~3KB per session | Database, testing, development |
| **Bash Script** | Any | Low-Medium | Zero | Automation, CI/CD, simple tasks |

### Decision Matrix:

**Use Slash Command When:**
- ✅ Operation is infrequent (weekly/monthly)
- ✅ Requires AI decision-making
- ✅ Needs context from multiple files
- ✅ Benefits from conversational interaction

**Use Always-Loaded MCP Server When:**
- ✅ Operation is frequent (daily/hourly)
- ✅ Core to development workflow
- ✅ Needs to be always available
- ✅ Benefits from persistent state

**Use Bash Script When:**
- ✅ Operation is fully automatable
- ✅ No AI decision-making needed
- ✅ Simple sequence of commands
- ✅ Used in CI/CD pipelines

---

## Quick Reference

| Command | Purpose | Frequency | Token Cost |
|---------|---------|-----------|------------|
| `/commands` | Show this guide | As needed | 0 until used |
| `/gitea-release` | Manage Gitea releases | Per release | 0 until used |

---

## Tips & Tricks

### 1. Chain Commands in Conversation
```
/gitea-release
"List all releases, then create v0.11.0 if it doesn't exist"
```

### 2. Combine with Scripts
```
/gitea-release
"I just ran the build script, now upload the artifacts"
```

### 3. Ask for Clarification
```
/gitea-release
"What information do you need to create a release?"
```

### 4. Reference Previous Work
```
/gitea-release
"Use the same format as v0.10.0 release"
```

---

## Troubleshooting

### Command Not Found
**Problem:** `/command-name` shows "command not found"

**Solution:**
- Check file exists: `.claude/commands/command-name.md`
- Verify file has `.md` extension
- Restart Claude Code session

### Command Loads Wrong Context
**Problem:** Command loads but provides incorrect information

**Solution:**
- Check command file content
- Verify file paths referenced are correct
- Update command file if project structure changed

### Want to Update a Command
**Solution:**
- Edit `.claude/commands/command-name.md`
- Changes take effect on next invocation
- No need to restart Claude Code

---

## Related Documentation

- **MCP Config**: `.claude/MCP-CONFIG.md` - Understanding MCP servers vs commands
- **Quick Reference**: `.claude/quick-refs/commands.md` - Bash command reference
- **Main Docs**: `CLAUDE.md` - Full project documentation
- **Onboarding**: `.claude/ONBOARDING.md` - New session quick start

---

## Contributing New Commands

If you create a useful command, document it here:

### How to Add:

1. Create `.claude/commands/your-command.md`
2. Follow the template above
3. Add entry to this reference file
4. Test the command
5. Update related documentation

### Command Ideas Welcome:

Open to suggestions! Common patterns that would benefit from slash commands:
- Deployment workflows
- Performance profiling
- Security audits
- Documentation generation
- Dependency updates

---

**Last Updated:** November 17, 2025
**Available Commands:** 2 (commands, gitea-release)
**Location:** `/Volumes/base/dev/e-fees/.claude/commands/`
