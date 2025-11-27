# New Feature Session Template

> **Session Type**: New Feature Development
> **Estimated Time**: 2-6 hours
> **Token Budget**: 50-80 KB
> **Difficulty**: High

---

## 📋 Pre-Session Checklist

Before starting, ensure you have:

- [ ] **Feature requirements** - Clear understanding of what to build
- [ ] **User stories** - Who needs this and why?
- [ ] **Acceptance criteria** - How will you know it's done?
- [ ] **Design mockups** - UI/UX approved (if applicable)
- [ ] **Technical approach** - High-level plan agreed upon
- [ ] **Dependencies identified** - What else needs to change?

---

## 🎯 Session Setup

### Step 1: Load Core Documentation

**Start with** (in order):
1. [NAVIGATOR.md](../NAVIGATOR.md) - Fast routing (3 KB)
2. [claude.md](../claude.md) - Project overview & patterns (10 KB)
3. [SUB-AGENT-ROUTING.md](../SUB-AGENT-ROUTING.md) - Identify specialists (8 KB)

**Total so far**: ~21 KB

### Step 2: Identify Required Specialists

Based on feature type, load ONE primary specialist:

```
Frontend feature → Frontend Specialist (18 KB)
Backend/IPC → Tauri Developer (17 KB)
Database → Database Specialist (19 KB)
Testing → Testing Specialist (21 KB)
```

**Running total**: ~35-40 KB

### Step 3: Load Context (as needed)

Only load what you need for this specific feature:

- [MCP Architecture](../context/mcp-architecture.md) - 18 KB - MCP integration
- [Database Schema](../context/database-schema.md) - 30 KB - New tables/queries
- [Testing Strategy](../context/testing-strategy.md) - 27 KB - E2E test patterns

**Budget check**: Stay under 80 KB total

---

## 📐 Planning Phase

### Architecture Review

**Use Code Reviewer** for this phase:

1. **Where does this fit?** - Which directory/module?
2. **What patterns to follow?** - Existing similar features?
3. **What will change?** - List affected files
4. **Dependencies?** - What else needs updating?

### Technical Design

Document your approach:

**Feature**: _[Name of feature]_

**Components Needed**:
- [ ] Frontend UI component(s)
- [ ] Tauri IPC command(s)
- [ ] Database query/schema
- [ ] E2E test(s)

**Files to Create**:
```
src/lib/components/[FeatureName].svelte
src-tauri/src/commands/[feature].rs
src-tauri/src/db/[feature]_queries.rs
e2e-mcp/tests/[feature].spec.ts
```

**Files to Modify**:
```
src-tauri/src/main.rs (register IPC)
src/routes/[route]/+page.svelte (integrate component)
```

**Estimated Scope**:
- New lines of code: _[estimate]_
- Files changed: _[count]_
- Test coverage: _[% or lines]_

---

## 🛠️ Implementation Phase

### Step 1: Database Layer (if needed)

**Delegate to Database Specialist**:

```bash
# 1. Update schema (if new tables)
# Document in .claude/context/database-schema.md

# 2. Create queries
# src-tauri/src/db/[feature]_queries.rs

# 3. Test queries manually
npm run db:studio
```

**Checklist**:
- [ ] Schema updated
- [ ] Queries follow Thing object patterns
- [ ] extractId() used correctly
- [ ] Queries tested in Surrealist

### Step 2: Backend/IPC (if needed)

**Delegate to Tauri Developer**:

```rust
// src-tauri/src/commands/[feature].rs

#[tauri::command]
pub async fn feature_command(
    state: State<'_, AppState>,
    param: Type,
) -> Result<ReturnType, String> {
    // Implementation
}
```

**Checklist**:
- [ ] Command created
- [ ] Command registered in main.rs
- [ ] Proper error handling
- [ ] Returns serializable types
- [ ] Tested with Tauri dev tools

### Step 3: Frontend (if needed)

**Delegate to Frontend Specialist**:

```svelte
<!-- src/lib/components/FeatureName.svelte -->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  // Component logic
</script>

<!-- Template -->
```

**Checklist**:
- [ ] Component created
- [ ] Proper Svelte 5 syntax (runes, etc.)
- [ ] DPI scaling considered (use rem, not px)
- [ ] Tailwind classes used correctly
- [ ] Responsive design
- [ ] Accessibility (ARIA labels, keyboard nav)

### Step 4: Integration

**Connect all layers**:

```
Database ←→ Tauri Command ←→ Frontend Component
```

**Test integration**:
```bash
# Start app
npm run tauri:dev

# Manually test the feature
# Check console for errors
# Verify data flow
```

### Step 5: E2E Testing

**Delegate to Testing Specialist**:

```typescript
// e2e-mcp/tests/[feature].spec.ts
import { test, expect } from '@playwright/test'

test.describe('Feature Name', () => {
  test('should do X', async ({ page }) => {
    // CRITICAL: Use "DELETE ME" prefix for all test data
    const testData = {
      name: `DELETE ME - Test ${Date.now()}`
    }

    // Test implementation
  })
})
```

**Checklist**:
- [ ] Happy path tested
- [ ] Edge cases tested
- [ ] Error handling tested
- [ ] "DELETE ME" prefix used
- [ ] Cleanup verified

---

## ✅ Completion Checklist

### Code Quality
- [ ] Follows existing patterns
- [ ] No code duplication
- [ ] Proper error handling
- [ ] Comments for complex logic
- [ ] No console.log/println left in code
- [ ] TypeScript types defined
- [ ] Rust code compiles without warnings

### Testing
- [ ] E2E test created and passing
- [ ] Manual testing complete
- [ ] All existing tests still pass
- [ ] Test data cleanup verified
- [ ] Edge cases covered

### Documentation
- [ ] Feature documented (if needed)
- [ ] Database schema updated (if changed)
- [ ] API documented (if new commands)
- [ ] User-facing changes noted

### Code Review Prep
- [ ] Run Code Reviewer specialist
- [ ] Fix any issues found
- [ ] Ensure consistent style
- [ ] Check for architectural concerns

### Git
- [ ] Changes committed in logical chunks
- [ ] Conventional commit messages
- [ ] Branch created from main/develop
- [ ] Ready for PR

---

## 📝 Commit Strategy

**Break into logical commits**:

```bash
# Commit 1: Database changes
git add src-tauri/src/db/
git commit -m "feat(db): add [feature] queries and schema"

# Commit 2: Backend/IPC
git add src-tauri/src/commands/
git add src-tauri/src/main.rs
git commit -m "feat(tauri): add [feature] IPC command"

# Commit 3: Frontend
git add src/lib/components/
git add src/routes/
git commit -m "feat(ui): add [feature] component and integration"

# Commit 4: Tests
git add e2e-mcp/tests/
git commit -m "test(e2e): add [feature] E2E tests"
```

---

## 🚨 Common Pitfalls

### Pitfall 1: Building Too Much at Once
**Problem**: Trying to implement entire feature in one session
**Solution**: Break into smaller milestones, commit incrementally

### Pitfall 2: Skipping Tests
**Problem**: "I'll add tests later"
**Solution**: Tests are part of the feature, not optional

### Pitfall 3: Ignoring DPI Scaling
**Problem**: UI looks good at 100% but breaks at 125%/150%
**Solution**: Use Frontend Specialist DPI patterns from the start

### Pitfall 4: Thing Object Mistakes
**Problem**: Treating SurrealDB IDs as strings
**Solution**: Always use extractId() helper

### Pitfall 5: Incomplete Error Handling
**Problem**: Happy path works, error paths crash
**Solution**: Test failure scenarios explicitly

---

## 🔄 Multi-Session Features

If feature requires multiple sessions:

### Session 1: Database & Backend (2-3 hours)
- [ ] Database schema
- [ ] Queries
- [ ] IPC commands
- [ ] Backend tests

### Session 2: Frontend (2-3 hours)
- [ ] UI components
- [ ] Integration with backend
- [ ] Styling
- [ ] Manual testing

### Session 3: Testing & Polish (1-2 hours)
- [ ] E2E tests
- [ ] Edge cases
- [ ] Error handling
- [ ] Code review

**Track progress**: Update `.claude/conversations/feature-[name].md`

---

## 📊 Session Metrics

Track these for estimation:

- **Planning time**: _[minutes]_
- **Implementation time**: _[minutes]_
- **Testing time**: _[minutes]_
- **Total token usage**: _[KB]_
- **Files changed**: _[count]_
- **Lines added/removed**: _[+XXX / -YYY]_
- **Test coverage**: _[%]_

**Total session time**: _[hours]_

---

## 🎯 Success Criteria

Feature is complete when:

- ✅ All acceptance criteria met
- ✅ E2E tests passing
- ✅ No regressions in existing tests
- ✅ Code reviewed (by Code Reviewer specialist)
- ✅ Documentation updated
- ✅ DPI scaling verified
- ✅ Error handling complete
- ✅ Test data cleanup verified

---

## 📚 Related Resources

- [Development Workflow](../rules/development-workflow.md) - Feature development rules
- [Database Patterns](../prompts/database-patterns.md) - Query best practices
- [Tauri Development](../prompts/tauri-development.md) - IPC patterns
- [Testing Strategy](../context/testing-strategy.md) - E2E approach

---

## 💡 Pro Tips

1. **Start with the test** - Write the E2E test first (TDD approach)

2. **Use parallel specialists** - Frontend and Backend can be developed in parallel

3. **Commit often** - Small, focused commits are easier to review

4. **DPI first** - Test at 125% scale from the beginning

5. **Thing objects** - When in doubt, use extractId()

6. **MCP socket** - Verify it exists before E2E tests

7. **Code review yourself** - Run Code Reviewer specialist before PR

8. **Token budget** - If hitting limits, use sub-agents more

---

**Happy building!** 🚀

Remember: Great features are well-tested, follow patterns, and solve real problems.
