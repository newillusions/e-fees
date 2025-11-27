# Claude Code Navigator

> **Quick Start**: Answer these questions to find the right documentation instantly
>
> **Purpose**: Fast routing to the right documentation based on your current task
>
> **Time to Navigate**: 10-30 seconds

---

## 🚀 Start Here: What Are You Trying to Do?

```
A. Fix something broken       → Go to Section 2: BROKEN
B. Build something new         → Go to Section 3: BUILD
C. Understand something        → Go to Section 4: LEARN
D. Review/improve code         → Go to Section 5: REVIEW
E. Set up development env      → Go to Section 6: SETUP
```

---

## 2. BROKEN: What's Not Working?

### Critical Issues (Fix Immediately)

| Symptom | Quick Fix | Detailed Help |
|---------|-----------|---------------|
| **App won't start** | `cargo clean && cargo build` | [Emergency Steps](claude.md#emergency-first-steps) |
| **Data corruption** | See recovery steps → | [Database Recovery](claude.md#database-recovery) |
| **Socket not found** | `pkill -f tauri-mcp && rm /tmp/tauri-mcp-e2e.sock` | [MCP Specialist](subagents/subagent-mcp-specialist.md) |
| **All tests failing** | Check socket, run cleanup | [Testing Specialist](subagents/subagent-testing-specialist.md) |

### By Error Domain

```
├─ Socket/MCP Errors
│  ├─ "Socket not found" → [MCP Specialist](subagents/subagent-mcp-specialist.md)
│  ├─ "Connection refused" → [MCP Servers](MCP-SERVERS.md#troubleshooting)
│  └─ Protocol errors → [MCP Architecture](context/mcp-architecture.md)
│
├─ Database Errors
│  ├─ Query failures → [Database Specialist](subagents/subagent-database-specialist.md)
│  ├─ Thing object issues → [Database Schema](context/database-schema.md#thing-objects)
│  └─ Connection issues → [Database Patterns](prompts/database-patterns.md)
│
├─ Build/Compilation Errors
│  ├─ Rust errors → [Tauri Developer](subagents/subagent-tauri-developer.md)
│  ├─ TypeScript errors → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
│  └─ Dependency issues → [Quick Troubleshooting](claude.md#quick-troubleshooting)
│
├─ Test Failures
│  ├─ E2E tests → [Testing Specialist](subagents/subagent-testing-specialist.md)
│  ├─ Test data issues → [Testing Strategy](context/testing-strategy.md)
│  └─ Flaky tests → [Testing Specialist](subagents/subagent-testing-specialist.md)
│
├─ UI/Display Issues
│  ├─ Layout broken → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
│  ├─ DPI scaling → [Frontend Specialist](subagents/subagent-frontend-specialist.md#dpi-scaling)
│  ├─ Styling issues → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
│  └─ Components not rendering → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
│
└─ Performance Issues
   ├─ Slow queries → [Database Specialist](subagents/subagent-database-specialist.md)
   ├─ UI lag → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
   └─ App responsiveness → [Code Reviewer](subagents/subagent-code-reviewer.md)
```

### Quick Diagnostic Commands

```bash
# Check app health
ps aux | grep -i "e-fees"

# Check MCP socket
ls -la /tmp/tauri-mcp-e2e.sock

# Check for test data
npm run test:e2e:list-test-data

# View recent logs
tail -50 src-tauri/target/debug/app.log
```

---

## 3. BUILD: What Are You Creating?

### New Features

**Start with**: [claude.md](claude.md) + [Routing Guide](SUB-AGENT-ROUTING.md)

```
Feature Type          → Load These Docs                    → Use This Specialist
─────────────────────────────────────────────────────────────────────────────
New Tauri command     → Tauri Development prompt           → Tauri Developer
New database table    → Database Schema                    → Database Specialist
New UI component      → Component patterns                 → Frontend Specialist
New E2E test          → Testing Strategy                   → Testing Specialist
New API endpoint      → Tauri Development + DB Schema      → Tauri Developer + Database
New form              → Frontend patterns + Validation     → Frontend Specialist
```

### Common Tasks Quick Links

| Task | Primary Doc | Specialist |
|------|-------------|------------|
| **Add IPC command** | [Tauri Development](prompts/tauri-development.md) | [Tauri Developer](subagents/subagent-tauri-developer.md) |
| **Add database query** | [Database Patterns](prompts/database-patterns.md) | [Database Specialist](subagents/subagent-database-specialist.md) |
| **Add UI component** | [Frontend Specialist](subagents/subagent-frontend-specialist.md) | Frontend Specialist |
| **Add E2E test** | [Testing Strategy](context/testing-strategy.md) | [Testing Specialist](subagents/subagent-testing-specialist.md) |
| **Add validation** | [Development Workflow](rules/development-workflow.md) | [Frontend](subagents/subagent-frontend-specialist.md) or [Tauri](subagents/subagent-tauri-developer.md) |

### Feature Development Workflow

```
1. Read: claude.md (overview)
2. Check: Routing guide (which specialist?)
3. Load: Relevant specialist file
4. Load: Context docs only if needed
5. Implement
6. Test
7. Review with Code Reviewer
```

---

## 4. LEARN: What Do You Need to Understand?

### Architecture & Concepts

```
What I need to understand    → Read This
────────────────────────────────────────────────────────────────
How MCP works                → [MCP Architecture](context/mcp-architecture.md)
Database structure           → [Database Schema](context/database-schema.md)
Testing approach             → [Testing Strategy](context/testing-strategy.md)
Project overview             → [ONBOARDING](ONBOARDING.md)
Development workflow         → [Development Workflow](rules/development-workflow.md)
```

### Technology-Specific

```
Technology               → Documentation
─────────────────────────────────────────────────────────────
Tauri v2                 → [Tauri Development](prompts/tauri-development.md)
SurrealDB                → [Database Schema](context/database-schema.md)
Svelte 5                 → [Frontend Specialist](subagents/subagent-frontend-specialist.md)
MCP Protocol             → [MCP Architecture](context/mcp-architecture.md)
E2E Testing              → [Testing Strategy](context/testing-strategy.md)
```

### Quick Concepts

| Concept | Where to Learn | Time to Read |
|---------|----------------|--------------|
| **"DELETE ME" pattern** | [Testing Strategy](context/testing-strategy.md#delete-me-pattern) | 2 min |
| **Thing objects** | [Database Schema](context/database-schema.md#thing-objects) | 3 min |
| **Sub-agent system** | [claude.md](claude.md#sub-agent-system) | 5 min |
| **MCP socket protocol** | [MCP Architecture](context/mcp-architecture.md) | 10 min |
| **IPC commands** | [Tauri Development](prompts/tauri-development.md) | 8 min |

---

## 5. REVIEW: What Needs Improvement?

### Code Quality Reviews

```
Review Type           → Use This Specialist       → Additional Context
─────────────────────────────────────────────────────────────────────
Architecture          → Code Reviewer             → Development Workflow
Performance           → Database Specialist       → Database Patterns
Test quality          → Testing Specialist        → Testing Strategy
UI/UX                 → Frontend Specialist       → Component patterns
Security              → Code Reviewer             → Development Workflow
```

### Specific Review Scenarios

| Scenario | Primary Reviewer | Secondary Reviewer |
|----------|-----------------|-------------------|
| **New feature PR** | [Code Reviewer](subagents/subagent-code-reviewer.md) | Domain specialist |
| **Database optimization** | [Database Specialist](subagents/subagent-database-specialist.md) | Code Reviewer |
| **UI component** | [Frontend Specialist](subagents/subagent-frontend-specialist.md) | Code Reviewer |
| **Test suite** | [Testing Specialist](subagents/subagent-testing-specialist.md) | Code Reviewer |
| **Refactoring** | [Code Reviewer](subagents/subagent-code-reviewer.md) | All specialists |

### Review Checklist

```bash
# Before requesting review, verify:
npm test                      # All tests pass
npm run test:e2e:verify-clean # No test data left
cargo check                   # Rust code compiles
npm run check                 # TypeScript type check
```

---

## 6. SETUP: Initial Environment Setup

### New Developer Onboarding

**Start here**: [ONBOARDING.md](ONBOARDING.md)

**Quick setup path**:
```
1. Read: ONBOARDING.md (5 min)
2. Run: Setup commands (10 min)
3. Verify: Health checks (2 min)
4. Read: claude.md overview (5 min)
Total: ~20 minutes
```

### Specific Setup Scenarios

```
Setup Need              → Documentation
───────────────────────────────────────────────────────────
New machine             → [ONBOARDING](ONBOARDING.md#first-time-setup)
New developer           → [ONBOARDING](ONBOARDING.md)
MCP servers             → [MCP-SERVERS](MCP-SERVERS.md)
Testing environment     → [Testing Strategy](context/testing-strategy.md)
Database                → [Database Schema](context/database-schema.md)
```

---

## 📊 Token Budget Guide

Choose the right documentation to load based on your session type:

### Session Type Matrix

| Session Type | Load (in order) | Expected Tokens | Time |
|-------------|-----------------|-----------------|------|
| **Quick fix** (<10min) | 1. Navigator<br>2. claude.md troubleshooting | 15-20 KB | 30s |
| **Bug fix** (30-60min) | 1. Navigator<br>2. Routing guide<br>3. Relevant specialist | 30-45 KB | 1-2min |
| **New feature** (2-4hr) | 1. Navigator<br>2. claude.md<br>3. Routing guide<br>4. Specialist<br>5. Context as needed | 50-80 KB | 2-3min |
| **Refactoring** (4-8hr) | 1. Navigator<br>2. Code Reviewer<br>3. Specialists as needed | 60-100 KB | 3-5min |
| **Learning** (1hr) | 1. Navigator<br>2. Relevant context docs | 30-50 KB | 2min |

### Loading Strategy

**DO**:
- ✅ Start with Navigator (this file)
- ✅ Load only what you need for current task
- ✅ Add context incrementally as needed
- ✅ Use specialists for deep dives

**DON'T**:
- ❌ Load all documentation upfront
- ❌ Load specialists you won't use
- ❌ Load context docs before knowing your task

---

## 🎯 Common Workflows

### 1. "I need to fix a bug"

```
Step 1: Identify domain (Section 2: BROKEN)
Step 2: Quick fix or detailed help?
Step 3: If quick fix works → Done
Step 4: If not → Load specialist
Step 5: Follow specialist guidance
```

**Estimated time**: 5-30 minutes

---

### 2. "I need to add a new feature"

```
Step 1: Read claude.md overview
Step 2: Check routing guide for specialist
Step 3: Load specialist file
Step 4: Load context docs only if needed
Step 5: Implement with specialist guidance
Step 6: Review with Code Reviewer
```

**Estimated time**: 2-6 hours

---

### 3. "I'm new to the project"

```
Step 1: Read ONBOARDING.md (5 min)
Step 2: Run setup commands (10 min)
Step 3: Read claude.md overview (5 min)
Step 4: Verify environment (2 min)
Step 5: Try first task with specialist
```

**Estimated time**: 20-30 minutes

---

### 4. "Tests are failing"

```
Step 1: Check quick diagnostic commands
Step 2: Identify failure type (socket, data, assertion)
Step 3: Load Testing Specialist
Step 4: If MCP-related → also load MCP Specialist
Step 5: Follow troubleshooting guide
```

**Estimated time**: 10-60 minutes

---

### 5. "I need to understand how X works"

```
Step 1: Find topic in Section 4: LEARN
Step 2: Load recommended documentation
Step 3: If still unclear → load specialist
Step 4: Use specialist for Q&A
```

**Estimated time**: 15-45 minutes

---

## 🔍 Quick Search

Can't find what you need? Try these approaches:

### By File/Directory Context

```
Working in...             → Use This Specialist
────────────────────────────────────────────────────────
src-tauri/                → Tauri Developer
src-tauri/src/db/         → Database Specialist
src/lib/components/       → Frontend Specialist
e2e-mcp/                  → Testing Specialist
tauri-plugin-mcp/         → MCP Specialist
src/routes/               → Frontend Specialist
```

### By Technology

```
Technology    → Start Here
─────────────────────────────────────────────────────
Rust          → Tauri Developer
TypeScript    → Frontend Specialist
Svelte        → Frontend Specialist
SurrealDB     → Database Specialist
MCP           → MCP Specialist
Testing       → Testing Specialist
```

### By Problem Symptom

```
Symptom                   → Likely Cause              → Specialist
───────────────────────────────────────────────────────────────────
Socket connection error   → MCP server not running    → MCP Specialist
Query returns Thing       → Missing extractId()       → Database Specialist
UI element clipped        → DPI scaling issue         → Frontend Specialist
Test leaves data          → Missing "DELETE ME"       → Testing Specialist
Build fails               → Dependency or syntax      → Tauri Developer
```

---

## 📚 Full Documentation Index

### Core Files (Always Available)
- [claude.md](claude.md) - Main entry point, troubleshooting
- [ONBOARDING.md](ONBOARDING.md) - Quick start guide
- [NAVIGATOR.md](NAVIGATOR.md) - This file

### Specialists (Load as Needed)
- [MCP Specialist](subagents/subagent-mcp-specialist.md) - 13.5 KB
- [Tauri Developer](subagents/subagent-tauri-developer.md) - 17.3 KB
- [Testing Specialist](subagents/subagent-testing-specialist.md) - 21 KB
- [Database Specialist](subagents/subagent-database-specialist.md) - 19 KB
- [Code Reviewer](subagents/subagent-code-reviewer.md) - 18.5 KB
- [Frontend Specialist](subagents/subagent-frontend-specialist.md) - 18.1 KB

### Context (Deep Dives)
- [MCP Architecture](context/mcp-architecture.md) - 18.2 KB
- [Testing Strategy](context/testing-strategy.md) - 27 KB
- [Database Schema](context/database-schema.md) - 29.6 KB

### Guides (Task-Specific)
- [Routing Guide](SUB-AGENT-ROUTING.md) - 17 KB
- [MCP Servers](MCP-SERVERS.md) - Configuration guide
- [Development Workflow](rules/development-workflow.md) - 12.5 KB

### Prompts (Advanced)
- [MCP Troubleshooting](prompts/mcp-troubleshooting.md) - 5 KB
- [Tauri Development](prompts/tauri-development.md) - 9 KB
- [Database Patterns](prompts/database-patterns.md) - 14 KB

---

## ⏱️ Estimated Reading Times

| Document | Read Time | When to Read |
|----------|-----------|--------------|
| Navigator (this) | 5 min | Start of every session |
| claude.md | 10 min | First session, then reference |
| ONBOARDING | 5 min | New developer only |
| Specialist file | 8-12 min | When needed for domain work |
| Context doc | 15-20 min | Deep dive into architecture |
| Prompt file | 5-8 min | Specific task guidance |
| Routing guide | 8 min | When unsure which specialist |

---

## 🎯 Pro Tips

1. **Always start here** - This Navigator saves time every session
2. **Load incrementally** - Don't load all docs upfront
3. **Use specialists liberally** - They're optimized for parallel work
4. **Check quick fixes first** - Many issues have 1-line solutions
5. **Follow token budget** - Stay within recommended loads

---

## 🆘 Still Lost?

If you've read this Navigator and still aren't sure where to go:

1. **Quick issue?** → Try [Quick Troubleshooting](claude.md#quick-troubleshooting)
2. **Complex task?** → Start with [Code Reviewer](subagents/subagent-code-reviewer.md) for triage
3. **New to project?** → Read [ONBOARDING](ONBOARDING.md) completely
4. **Unsure of specialist?** → Use [Routing Guide](SUB-AGENT-ROUTING.md)

---

**Last Updated**: November 6, 2025
**Maintained By**: Martin & Claude Code
**Feedback**: Add suggestions to `.claude/conversations/`
