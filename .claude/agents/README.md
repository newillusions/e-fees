# Claude Code Native Agents (e-fees)

Project-specific specialist agents. Claude auto-delegates to one when a task matches its `description`, or you can invoke it explicitly via the Task tool. Each runs in an isolated context with its own tool access.

## Roster

| Agent | Use for |
|-------|---------|
| **database-specialist** | SurrealDB queries/schema, v3 gotchas, query optimization, DB connection debugging |
| **tauri-developer** | Rust/Tauri v2 commands, app state, filesystem/native integration, the standalone API + scope service |
| **testing-specialist** | E2E (Tauri-MCP) scenarios, `DELETE ME` test-data hygiene, test-failure debugging |
| **mcp-specialist** | Tauri-MCP socket/protocol issues, E2E harness failures |
| **code-reviewer** | Code quality, project-convention adherence, refactoring (review-only, no Bash writes) |

All are `model: sonnet`. They carry only e-fees-specific knowledge; general engineering, TDD discipline, and the full project context live in `../../CLAUDE.md` and `.claude/rules/development-workflow.md`.

## File format

```markdown
---
name: agent-name            # lowercase, hyphens
description: When to use this agent (drives auto-delegation)
tools: [Bash, Read, Write, Edit, Grep]   # optional restriction
model: sonnet               # optional override
---

# System prompt (markdown body) — keep it project-specific and tight
```

## Notes
- TDD is mandatory for all coding work and is owned by CLAUDE.md (Critical Directives) + the superpowers plugin — not by a separate agent.
- Frontend/UI work and general implementation are done in the main session; there is no separate frontend agent.
