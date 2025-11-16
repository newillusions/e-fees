# Claude Code Native Agents

This directory contains **Claude Code native agents** with automatic delegation based on task matching.

## How Native Agents Work

Unlike the old manual approach (archived in `../subagents/archived/`), these agents:

- ✅ **Auto-delegate** based on task descriptions and context
- ✅ **Separate context windows** preventing pollution of main conversations
- ✅ **Configurable tool access** per agent
- ✅ **CLI integration** via `/agents` command

## File Format

Each agent uses **YAML frontmatter + Markdown**:

```markdown
---
name: agent-name           # lowercase, hyphens only
description: Brief purpose and when to use this agent
tools: [Bash, Read, Write, Edit, Grep]  # Optional tool restrictions
model: sonnet              # Optional model override
---

# Agent system prompt content here
```

## Available Agents

| Agent | Purpose | Key Responsibilities |
|-------|---------|---------------------|
| **mcp-specialist** | MCP socket & protocol debugging | Socket health, tool registration, E2E test MCP failures |
| **tauri-developer** | Rust backend & desktop integration | Tauri commands, IPC, file system operations, window management |
| **testing-specialist** | E2E testing & test data management | "DELETE ME" pattern, test scenarios, test cleanup |
| **database-specialist** | SurrealDB queries & schema | Thing objects, query optimization, schema design |
| **code-reviewer** | Code quality & architecture | Code smells, refactoring, best practices, consistency |
| **frontend-specialist** | Svelte UI & DPI scaling | Components, Tailwind CSS, responsive design, DPI fixes |

## Usage

### Automatic Delegation (Recommended)

Claude automatically invokes agents when tasks match their descriptions:

```bash
# Just describe the problem - Claude routes automatically
"The MCP socket isn't connecting"  → mcp-specialist
"Add a new Tauri command"          → tauri-developer
"Fix the DPI scaling on this UI"  → frontend-specialist
```

### Manual Invocation

Use the Task tool with agent name:

```markdown
<Task agent="mcp-specialist">
Debug why E2E tests can't connect to /tmp/tauri-mcp-e2e.sock
</Task>
```

### CLI Management

```bash
# List all agents
/agents list

# View agent details
/agents show mcp-specialist

# Test agent
/agents test database-specialist "optimize this query"
```

## Comparison: Old vs New

### ❌ Old Manual Approach (Archived)

Located in `../subagents/archived/subagent-*.md`

**Invocation:**
```markdown
**Sub-Agent Task: MCP Specialist**
Context: Brief context
[Include relevant subagent file: .claude/subagents/subagent-mcp-specialist.md]
```

**Problems:**
- Manual file loading into context
- No automatic delegation
- Pollutes main conversation context
- No tool restrictions per agent

### ✅ New Native Approach (Current)

Located in `.claude/agents/*.md`

**Invocation:**
```markdown
Automatic based on task matching, or:
<Task agent="mcp-specialist">task description</Task>
```

**Benefits:**
- Automatic delegation
- Isolated context windows
- Tool access control
- CLI integration
- Plugin support

## Migration Notes

**Old subagent files migrated on:** November 16, 2025

**Changes made:**
1. Added YAML frontmatter to each agent
2. Renamed files (removed `subagent-` prefix)
3. Updated tool lists in frontmatter
4. Moved originals to `../subagents/archived/`

**Backward compatibility:**
- Old files preserved in `../subagents/archived/`
- CLAUDE.md updated to reference new system
- SUB-AGENT-ROUTING.md still valid (concepts apply)

## Adding New Agents

1. Create `agent-name.md` in this directory
2. Use lowercase letters and hyphens only
3. Add required YAML frontmatter:
   ```yaml
   ---
   name: agent-name
   description: Purpose and when to use (max 1024 chars)
   tools: [optional-tool-list]
   ---
   ```
4. Write the system prompt as markdown body
5. Test with `/agents test agent-name "test task"`

## Related Documentation

- **Native Agent Docs**: https://code.claude.com/docs/en/sub-agents.md
- **Skills System**: `../ skills/` (reusable agent skills)
- **Slash Commands**: `../commands/` (project commands)
- **Old System Reference**: `../subagents/archived/` (legacy manual approach)
- **Routing Guide**: `../SUB-AGENT-ROUTING.md` (decision framework still applicable)

---

**Last Updated**: November 16, 2025
**Format**: Claude Code Native Agents (YAML frontmatter + Markdown)
