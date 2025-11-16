# E-Fees Onboarding Guide
**Quick-Start for Claude Code Sessions**

Welcome! This guide will get you oriented in the e-fees project quickly.

---

## 🎯 Project Overview

**E-Fees** is a desktop fee management application for tracking contacts, companies, invoices, and projects.

**Tech Stack**:
- **Frontend**: Svelte 5 + TypeScript
- **Backend**: Tauri v2 (Rust)
- **Database**: SurrealDB (embedded)
- **Testing**: Custom MCP integration for AI-driven E2E tests

**Project Root**: `/Volumes/base/dev/e-fees/`

---

## 🚀 First Steps (Do This First!)

### 1. Read This File First
You're already doing this! ✅

### 2. Check Current Status
```bash
# See what's in progress
cat .claude/conversations/current-tasks.md

# Check recent changes
git log --oneline -10
```

### 3. Understand the Structure
```
e-fees/
├── .claude/                    # ⭐ Claude Code configuration
│   ├── ONBOARDING.md          # ← You are here
│   ├── claude.md              # Quick reference & troubleshooting
│   ├── subagents/             # Specialized sub-agents (MANDATORY USE)
│   ├── prompts/               # Detailed task-specific prompts
│   ├── rules/                 # Development workflow rules
│   ├── context/               # Architecture & strategy docs
│   └── conversations/         # Task tracking & session history
├── src/                       # Svelte frontend
├── src-tauri/                 # Rust backend (Tauri)
├── e2e-mcp/                   # E2E tests with MCP
└── tauri-plugin-mcp/          # Custom MCP plugin
```

---

## 📋 Quick Reference

### Essential Commands

```bash
# Development
npm run tauri:dev              # Start dev server
cargo test                     # Run Rust tests
npm run test:e2e              # Run E2E tests

# Building
npm run tauri:build           # Production build

# Testing
npm test                      # All tests
npm run test:e2e:verify-clean # Check for leftover test data

# Database
# SurrealDB runs embedded - no separate server needed
```

### Key Directories

| Path | Purpose |
|------|---------|
| `src/lib/components/` | Svelte UI components |
| `src/lib/types/` | TypeScript type definitions |
| `src-tauri/src/commands/` | Tauri IPC commands |
| `src-tauri/src/db/` | Database queries |
| `e2e-mcp/src/tests/` | E2E test suites |

---

## 🧠 Sub-Agent System (MANDATORY)

**CRITICAL**: You MUST use sub-agents for specialized tasks. Do NOT handle these in the main conversation.

### When to Delegate

| If you're working on... | Use this sub-agent | Location |
|------------------------|-------------------|----------|
| MCP issues, socket errors | **MCP Specialist** | `.claude/subagents/subagent-mcp-specialist.md` |
| Tauri commands, IPC | **Tauri Developer** | `.claude/subagents/subagent-tauri-developer.md` |
| E2E tests, test data cleanup | **Testing Specialist** | `.claude/subagents/subagent-testing-specialist.md` |
| Database queries, schema | **Database Specialist** | `.claude/subagents/subagent-database-specialist.md` |
| Code reviews, quality checks | **Code Reviewer** | `.claude/subagents/subagent-code-reviewer.md` |

### How to Use Sub-Agents

```markdown
# In main conversation:
"I need to add a new Tauri command for contact search. 
Let me delegate this to the Tauri Developer sub-agent."

# Then open: .claude/subagents/subagent-tauri-developer.md
# Follow the prompt to complete the task
# Return to main conversation with results
```

**Why This Matters**: Sub-agents prevent token bloat and enable parallel workflows.

---

## 🔍 Common Tasks & Where to Start

### Adding a New Feature

1. **Read**: `.claude/rules/development-workflow.md` (Rule 11)
2. **Create branch**: `git checkout -b feat/your-feature`
3. **Use appropriate sub-agent** (see table above)
4. **Follow**: Testing requirements (Rule 5-7)
5. **Commit**: Using conventional commits (Rule 2)

### Fixing a Bug

1. **Read**: `.claude/rules/development-workflow.md` (Rule 13)
2. **Create failing test** that demonstrates bug
3. **Create branch**: `git checkout -b fix/bug-description`
4. **Fix and verify** all tests pass
5. **Add regression test**

### MCP Troubleshooting

1. **Delegate to**: `.claude/subagents/subagent-mcp-specialist.md`
2. **Reference**: `.claude/prompts/mcp-troubleshooting.md`
3. **Check**: Socket path `/tmp/tauri-mcp-e2e.sock`
4. **Verify**: Tauri app is running

### Database Work

1. **Delegate to**: `.claude/subagents/subagent-database-specialist.md`
2. **Reference**: `.claude/prompts/database-patterns.md`
3. **Follow**: Query optimization guidelines (Rule 18)
4. **Schema docs**: `.claude/context/database-schema.md`

---

## 📚 Documentation Hierarchy

### Start Here (Quick)
- ✅ **This file** - Onboarding overview
- ✅ `.claude/claude.md` - Quick troubleshooting & references

### Then Read (Rules)
- `.claude/rules/development-workflow.md` - 22 mandatory rules
- `.claude/rules/security-rules.md` - Security practices
- `.claude/rules/performance-rules.md` - Performance guidelines

### Deep Dives (Context)
- `.claude/context/mcp-architecture.md` - MCP integration details
- `.claude/context/testing-strategy.md` - E2E testing approach
- `.claude/context/database-schema.md` - Schema reference

### Task-Specific (Prompts)
- `.claude/prompts/mcp-troubleshooting.md` - MCP debugging
- `.claude/prompts/tauri-development.md` - Tauri patterns
- `.claude/prompts/database-patterns.md` - SurrealDB best practices

---

## 🎨 Development Patterns

### "DELETE ME" Pattern (CRITICAL)

**ALL test data MUST include "DELETE ME" prefix:**

```typescript
// ✅ CORRECT
const testContact = {
  name: `DELETE ME - Test Contact ${Date.now()}`,
  email: `delete-me-${Date.now()}@example.com`
}

// ❌ WRONG - Will fail code review
const testContact = {
  name: "Test Contact",
  email: "test@example.com"
}
```

**Why**: Ensures test data is identifiable and can be cleaned up automatically.

### Conventional Commits

```bash
feat(contacts): add phone validation
fix(mcp): resolve socket timeout
test(e2e): add company workflow
docs(readme): update setup instructions
```

### File Modifications

**Use targeted edits, not full file replacement:**

```bash
# ✅ GOOD - Edit specific section
str_replace old="function foo() {" new="function foo(param: string) {"

# ❌ BAD - Rewrite entire file
create_file path="existing-file.ts" content="..."
```

---

## 🐛 Troubleshooting

### Quick Diagnostics

```bash
# Check if services are running
ps aux | grep tauri           # Tauri app
ps aux | grep surreal         # SurrealDB (should be embedded)

# Check MCP socket
ls -la /tmp/tauri-mcp-e2e.sock

# View recent logs
tail -f src-tauri/target/debug/app.log
```

### Common Issues

| Problem | Solution |
|---------|----------|
| MCP socket not found | Start Tauri app first: `npm run tauri:dev` |
| Tests leaving data | Use "DELETE ME" prefix in all test data |
| Build fails | Clear caches: `rm -rf target/ node_modules/` then rebuild |
| Type errors | Regenerate types: `npm run tauri:generate-types` |

### Where to Get Help

1. **Check**: `.claude/claude.md` - Quick troubleshooting
2. **Search**: `.claude/conversations/` - Previous solutions
3. **Delegate**: Appropriate sub-agent for specialized issues

---

## ⚡ Token Efficiency Tips

### DO
- ✅ Read only relevant documentation sections
- ✅ Use sub-agents for specialized tasks
- ✅ Reference file paths instead of reading entire files
- ✅ Use targeted edits with `str_replace`
- ✅ Check `.claudeignore` to understand what's excluded

### DON'T
- ❌ Read entire large files unnecessarily
- ❌ Handle specialized tasks in main conversation
- ❌ Rewrite entire files when editing small sections
- ❌ Load build artifacts (they're ignored anyway)

---

## 🎯 Your Mission

### Immediate Next Steps

1. **Check current tasks**: `cat .claude/conversations/current-tasks.md`
2. **Review recent work**: `git log --oneline -5`
3. **Identify task type**: Determine which sub-agent to use
4. **Read relevant rules**: From `.claude/rules/` if needed
5. **Delegate or execute**: Use sub-agent or proceed in main conversation

### Session Goals

- ✅ Complete assigned tasks efficiently
- ✅ Follow all development rules
- ✅ Use sub-agents appropriately
- ✅ Write tests for all changes
- ✅ Document as you go
- ✅ Commit with conventional format

---

## 📊 Project Status

### Current Phase
**Week 2**: Documentation & context optimization

### Completed
- ✅ Week 1 optimizations (40 KB docs, 40-60% token savings expected)
- ✅ Sub-agent system implemented
- ✅ Development workflow rules established

### In Progress
- 🔄 Context documentation (architecture, testing, schema)
- 🔄 Rule refinements (security, performance)

### Upcoming
- Week 3: Code templates & snippets
- Week 4: Advanced sub-agent personas

---

## 🔗 Critical Resources

### Must-Read First
1. **This file** - ONBOARDING.md
2. `.claude/claude.md` - Quick reference
3. `.claude/rules/development-workflow.md` - Development rules

### Frequently Referenced
- `.claude/subagents/` - Sub-agent prompts
- `.claude/prompts/` - Task-specific guides
- `.claude/context/` - Architecture docs

### Emergency Reference
- `.claude/claude.md` - Troubleshooting section
- `.claude/prompts/mcp-troubleshooting.md` - MCP issues

---

## 📝 Session Checklist

Before ending your session:

- [ ] All tests passing (`npm test`)
- [ ] No test data left in database (`npm run test:e2e:verify-clean`)
- [ ] Changes committed with conventional format
- [ ] Documentation updated if needed
- [ ] `.claude/conversations/current-tasks.md` updated
- [ ] Sub-agent work documented in appropriate conversation files

---

## 💡 Pro Tips

1. **Always check current tasks first** - Avoid duplicate work
2. **Use sub-agents liberally** - They're designed for token efficiency
3. **Test early, test often** - Catch issues before they compound
4. **Document as you code** - Future you (and Claude) will thank you
5. **Keep commits atomic** - One logical change per commit

---

## 🆘 Getting Unstuck

If you're unsure what to do:

1. **Check**: `.claude/conversations/current-tasks.md`
2. **Review**: Recent git commits for context
3. **Read**: Relevant sub-agent or prompt file
4. **Ask**: "What was I working on?" - Check conversation files
5. **Delegate**: When in doubt, use appropriate sub-agent

---

**Welcome to E-Fees Development!**

You're now ready to contribute effectively. Remember:
- 🧠 **Use sub-agents** for specialized work
- 📋 **Follow the rules** in `.claude/rules/`
- 🧪 **Test everything** with "DELETE ME" patterns
- 💬 **Document progress** in conversation files

**Good luck, and happy coding!** 🚀

---

**Last Updated**: October 26, 2025  
**Version**: 1.0  
**Maintained By**: Martin & Claude Code
