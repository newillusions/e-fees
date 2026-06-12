# Bug Fix Session Template

> **Session Type**: Bug Fix
> **Estimated Time**: 30-90 minutes
> **Token Budget**: 30-45 KB
> **Difficulty**: Medium

---

## 📋 Pre-Session Checklist

Before starting, ensure you have:

- [ ] **Bug description** - Clear understanding of what's broken
- [ ] **Reproduction steps** - Can you reproduce the issue?
- [ ] **Expected vs actual behavior** - What should happen vs what happens
- [ ] **Error messages/logs** - Any console errors or log output
- [ ] **Recent changes** - What was changed before the bug appeared?

---

## 🎯 Session Setup

### Step 1: Load Core Documentation

**Start with** (in order):
1. [NAVIGATOR.md](../NAVIGATOR.md) - Fast routing (3 KB)
2. [claude.md](../claude.md) - Quick troubleshooting section (5-8 KB)

**Total so far**: ~10 KB

### Step 2: Identify Domain

Use the NAVIGATOR to determine which specialist to load:

```
├─ Socket/MCP errors → MCP Specialist (13.5 KB)
├─ Database errors → Database Specialist (19 KB)
├─ Build errors → Tauri Developer (17 KB)
├─ Test failures → Testing Specialist (21 KB)
├─ UI/Display issues → Frontend Specialist (18 KB)
└─ Unsure → Code Reviewer (18.5 KB) for triage
```

**Load**: ONE specialist file only (~15-20 KB)

**Running total**: ~25-30 KB

### Step 3: Load Context (if needed)

Only load if the specialist recommends it:

- [MCP Architecture](../context/mcp-architecture.md) - 18 KB - Only for MCP issues
- [Database Schema](../context/database-schema.md) - 30 KB - Only for DB queries
- [Testing Strategy](../context/testing-strategy.md) - 27 KB - Only for test issues

**Final budget**: 30-45 KB (within target)

---

## 🔍 Investigation Phase

### Quick Diagnostics

Run these commands to gather information:

```bash
# Check if app is running
ps aux | grep -i "e-fees"

# Check MCP socket (if MCP-related)
ls -la /tmp/tauri-mcp-e2e.sock

# View recent logs
tail -50 src-tauri/target/debug/app.log

# Check for test data contamination
npm run test:e2e:list-test-data

# Git blame (when was this code last changed?)
git log -p --follow [file-path]
```

### Gather Evidence

Document findings:

**Symptom**: _[What's happening?]_

**Error Messages**:
```
[Paste exact error messages]
```

**Reproduction**: _[Steps to reproduce]_

**Context**: _[What changed recently?]_

---

## 🛠️ Fix Implementation

### Step 1: Create Failing Test (MANDATORY)

**Before fixing**, write a test that demonstrates the bug:

```bash
# Create test file in appropriate location
# e2e-mcp/tests/regression/bug-[issue-number].spec.ts

# Test should:
# 1. Use "DELETE ME" prefix for test data
# 2. Reproduce the exact bug
# 3. FAIL before the fix
# 4. PASS after the fix
```

### Step 2: Implement Fix

Work with your chosen specialist to implement the fix.

**Remember**:
- ✅ Small, focused changes
- ✅ Follow existing patterns
- ✅ Add comments explaining the fix
- ❌ Don't refactor unrelated code
- ❌ Don't fix multiple bugs at once

### Step 3: Verify Fix

```bash
# Run the specific test
npm run test:e2e -- --grep "your-test-name"

# Run all tests to ensure no regressions
npm test

# Start the app and manually verify
npm run tauri:dev

# Clean up test data
npm run test:e2e:cleanup
npm run test:e2e:verify-clean
```

---

## ✅ Completion Checklist

Before closing the session:

### Code Quality
- [ ] Fix is minimal and focused
- [ ] No unrelated changes
- [ ] Code follows project patterns
- [ ] Comments explain WHY, not WHAT

### Testing
- [ ] Regression test created
- [ ] Test fails before fix
- [ ] Test passes after fix
- [ ] All other tests still pass
- [ ] No test data left behind

### Documentation
- [ ] Updated relevant docs (if needed)
- [ ] Added TODO comments for follow-ups (if needed)
- [ ] Logged the bug cause in conversation history

### Git
- [ ] Changes committed with conventional format
- [ ] Commit message explains the bug and fix
- [ ] Regression test committed with fix

---

## 📝 Commit Message Template

```bash
git commit -m "fix(domain): brief description of bug

Fixes issue where [describe problem].

Root cause: [explain what was wrong]
Solution: [explain the fix]

- Add regression test for [scenario]
- [Other changes]

Closes #[issue-number]"
```

---

## 🚨 Common Pitfalls

### Pitfall 1: Fixing Symptoms, Not Root Cause
**Problem**: Quick fix that doesn't address underlying issue
**Solution**: Use specialist to understand the architecture first

### Pitfall 2: No Regression Test
**Problem**: Bug will come back
**Solution**: ALWAYS create a failing test first

### Pitfall 3: Scope Creep
**Problem**: "While I'm here, let me also..."
**Solution**: Stay focused. Create separate issues for other improvements

### Pitfall 4: Breaking Other Things
**Problem**: Fix one thing, break another
**Solution**: Run full test suite before committing

### Pitfall 5: Test Data Contamination
**Problem**: Forgetting "DELETE ME" prefix
**Solution**: Use Testing Specialist checklist

---

## 📊 Session Metrics

Track these to improve over time:

- **Time to identify root cause**: _[minutes]_
- **Time to implement fix**: _[minutes]_
- **Number of test runs**: _[count]_
- **Token usage**: _[KB]_
- **Files changed**: _[count]_

**Total session time**: _[minutes]_

---

## 🔄 If Bug Isn't Fixed

### Escalation Path

1. **Review with Code Reviewer** - Get second opinion on approach
2. **Load additional context** - May need deeper architectural understanding
3. **Check git history** - When did this last work?
4. **Ask for help** - Document what you've tried, create detailed issue

### When to Stop

Stop and regroup if:
- ❌ Spent >90 minutes without progress
- ❌ Token budget exceeded (>50 KB)
- ❌ Breaking more things than fixing
- ❌ Not understanding the architecture

**Better to pause, research, and resume than to thrash**

---

## 📚 Related Resources

- [Quick Troubleshooting](../claude.md#quick-troubleshooting) - Fast fixes
- [SUB-AGENT-ROUTING](../SUB-AGENT-ROUTING.md) - Which specialist?
- [Development Workflow](../rules/development-workflow.md) - Process rules
- [NAVIGATOR](../NAVIGATOR.md) - Find documentation

---

## 💡 Pro Tips

1. **Git bisect is your friend** - When did this break?
   ```bash
   git bisect start
   git bisect bad  # Current state is broken
   git bisect good [commit-hash]  # This commit was working
   ```

2. **Minimal reproduction** - Simplest case that shows the bug

3. **Console.log/println debugging** - Don't be afraid of print statements

4. **Check for Thing objects** - Database bugs often involve Thing extraction

5. **Verify MCP socket first** - Many issues are just socket connectivity

---

**Good luck with your bug fix!** 🐛🔨

Remember: The best bug fix is one that's well-tested, minimal, and easy to understand.
