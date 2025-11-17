# E-Fees Slash Commands

This directory contains custom slash commands for the E-Fees project.

## What Are Slash Commands?

Slash commands are custom prompts that can be invoked in Claude Code by typing `/command-name`. They provide:

- **Zero token cost** until invoked
- **On-demand context** loading
- **Repeatable workflows** for common tasks
- **Self-documenting** project operations

## Quick Start

### View All Commands
```
/commands
```
This shows the comprehensive command reference with syntax, examples, and usage patterns.

### Available Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `/commands` | Command reference guide | Learning available commands |
| `/gitea-release` | Gitea release management | Creating/managing releases |

## Using Slash Commands

### Basic Usage
```
You: /gitea-release
Claude: [Context loaded - ready for release management]
You: "Create a release for v0.11.0"
Claude: [Executes with loaded context]
```

### Chaining Operations
```
You: /gitea-release
You: "List all releases, then create v0.11.0 if it doesn't exist and upload the DMG"
Claude: [Executes multi-step workflow]
```

## Command File Structure

Each command is a Markdown file in this directory:

```
.claude/commands/
  ├── README.md           # This file
  ├── commands.md         # Command reference (/commands)
  └── gitea-release.md    # Gitea release management (/gitea-release)
```

### File Naming
- Filename: `command-name.md`
- Invocation: `/command-name`
- Example: `gitea-release.md` → `/gitea-release`

## Creating New Commands

### 1. Create Command File
```bash
touch .claude/commands/my-command.md
```

### 2. Use Template
See `/commands` for the full template, or use this minimal structure:

```markdown
# My Command

Brief description of what this command does.

## What You Should Do

When this command is invoked, help the user with [specific task].

## Available Operations

1. **Operation 1** - Description
2. **Operation 2** - Description

## Example Usage

\`\`\`bash
# Example commands
command here
\`\`\`
```

### 3. Test Command
```
/my-command
"Test request"
```

### 4. Document It
Add entry to `commands.md` and this README.

## Design Philosophy

### Commands vs MCP Servers

**Use Slash Command for:**
- Infrequent operations (weekly/monthly)
- High-context, complex workflows
- Operations requiring AI decision-making
- Admin/maintenance tasks

**Use MCP Server for:**
- Frequent operations (daily/hourly)
- Core development workflows
- Always-needed functionality
- Real-time integrations

**Example:**
- ✅ `/gitea-release` - Used once per release (command)
- ✅ SurrealDB MCP - Used constantly during development (MCP server)

### Token Optimization

Commands are part of the project's token optimization strategy:

1. **Not loaded by default** - Zero cost when not in use
2. **Loaded on-demand** - Only when explicitly invoked
3. **Focused context** - Only relevant information
4. **Reusable** - Consistent workflows across sessions

See `.claude/MCP-CONFIG.md` for detailed discussion of this pattern.

## Command Categories

### Current
- **Project Management**: `/commands`
- **Release Management**: `/gitea-release`

### Planned
- **Testing**: `/test` - E2E test operations
- **Database**: `/db` - SurrealDB operations
- **Build**: `/build` - Multi-platform builds
- **MCP**: `/mcp` - MCP troubleshooting
- **Specialist**: `/specialist` - Agent routing

## Best Practices

### Do's ✅
- Keep commands focused on one domain
- Provide clear examples
- Reference relevant files/scripts
- Include troubleshooting tips
- Explain when NOT to use the command

### Don'ts ❌
- Don't create commands for frequent operations
- Don't duplicate MCP server functionality
- Don't make commands too generic
- Don't forget to document

## Examples

### Good Command Usage

```
# Infrequent, complex operation
/gitea-release
"Create v0.11.0 with release notes from CHANGELOG.md"

# Multi-step workflow
/build
"Build for all platforms and prepare distribution packages"
```

### Bad Command Usage

```
# Frequent operation - should be MCP server or direct command
/database-query
"SELECT * FROM contacts"

# Simple operation - should be direct command
/git-status
"Show me git status"
```

## Related Documentation

- **Command Reference**: Type `/commands` in Claude Code
- **MCP Configuration**: `.claude/MCP-CONFIG.md`
- **Project Docs**: `CLAUDE.md`
- **Quick Reference**: `.claude/quick-refs/commands.md`

---

**Directory**: `/Volumes/base/dev/e-fees/.claude/commands/`
**Current Commands**: 2
**Last Updated**: November 17, 2025
