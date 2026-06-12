# Code Review Session Template

> **Session Type**: Code Review
> **Estimated Time**: 30-60 minutes
> **Token Budget**: 25-40 KB
> **Difficulty**: Medium

---

## 📋 Pre-Session Checklist

Before starting review:

- [ ] **Code is complete** - All changes implemented
- [ ] **Tests are passing** - No failing tests
- [ ] **Author self-reviewed** - First pass already done
- [ ] **Scope is clear** - Know what's being reviewed
- [ ] **Context available** - PR description or commit messages

---

## 🎯 Session Setup

### Load Documentation

**Required**:
1. [NAVIGATOR.md](../NAVIGATOR.md) - Quick routing (3 KB)
2. [Code Reviewer Specialist](../subagents/subagent-code-reviewer.md) - Review expertise (18.5 KB)
3. [Development Workflow](../rules/development-workflow.md) - Project standards (12.5 KB)

**Total**: ~34 KB

**Optional** (load as needed):
- Domain specialist for technical review
- Context docs for architecture verification

---

## 🔍 Review Process

### Phase 1: High-Level Review (5-10 minutes)

**Questions to answer**:

1. **Purpose**: What is this code trying to achieve?
   - [ ] Clear from PR/commit description
   - [ ] Solves stated problem
   - [ ] No scope creep

2. **Architecture**: Does it fit the design?
   - [ ] Follows existing patterns
   - [ ] Doesn't duplicate code
   - [ ] Appropriate abstraction level
   - [ ] No architectural violations

3. **Scope**: Is the change size appropriate?
   - [ ] Not too large (>500 lines needs splitting)
   - [ ] Logically grouped
   - [ ] Related changes only

**Red Flags** 🚩:
- Mixing refactoring with feature work
- Touching unrelated files
- Large files with small changes
- No tests for new code

---

### Phase 2: Code Quality Review (15-20 minutes)

**Use Code Reviewer Specialist checklist**:

#### TypeScript/JavaScript
- [ ] Proper types (no `any`)
- [ ] Null/undefined handling
- [ ] Error handling
- [ ] No console.log in production code
- [ ] Async/await used correctly
- [ ] Imports organized

#### Rust
- [ ] Proper error handling (Result<T, E>)
- [ ] No unwrap() in production
- [ ] Lifetime annotations correct
- [ ] No compiler warnings
- [ ] Cargo fmt applied
- [ ] Clippy suggestions addressed

#### Svelte
- [ ] Svelte 5 syntax (runes, not legacy)
- [ ] Proper reactivity
- [ ] No memory leaks
- [ ] Event handlers cleaned up
- [ ] Proper use of $state, $derived, $effect

#### Database
- [ ] Thing objects handled correctly
- [ ] extractId() used where needed
- [ ] Queries optimized
- [ ] Proper indexes (if new tables)
- [ ] No N+1 queries

---

### Phase 3: Testing Review (10-15 minutes)

**Test Quality**:
- [ ] Tests exist for new functionality
- [ ] Tests use "DELETE ME" prefix
- [ ] Happy path covered
- [ ] Edge cases covered
- [ ] Error cases covered
- [ ] Tests are maintainable
- [ ] Test cleanup verified

**E2E Tests** (if applicable):
- [ ] MCP socket verified before tests
- [ ] Proper wait strategies
- [ ] No hard-coded waits
- [ ] Stable (not flaky)
- [ ] Data cleanup after test

**Test Coverage**:
```bash
# Check coverage
npm run test:coverage

# Should be >80% for new code
```

---

### Phase 4: Security & Performance (5-10 minutes)

**Security Checklist**:
- [ ] No hardcoded credentials
- [ ] Input validation present
- [ ] SQL injection prevented (use params)
- [ ] XSS prevented (proper escaping)
- [ ] No secrets in logs
- [ ] Proper authentication/authorization

**Performance Checklist**:
- [ ] No unnecessary re-renders
- [ ] Database queries efficient
- [ ] No blocking operations on main thread
- [ ] Large lists virtualized
- [ ] Images optimized

---

### Phase 5: UI/UX Review (if applicable)

**Frontend Specialist considerations**:

- [ ] **DPI Scaling**: Tested at 100%, 125%, 150%, 175%
- [ ] **Responsive**: Works at different window sizes
- [ ] **Accessibility**:
  - ARIA labels present
  - Keyboard navigation works
  - Color contrast sufficient
  - Screen reader friendly
- [ ] **Loading States**: Proper feedback during async ops
- [ ] **Error States**: User-friendly error messages
- [ ] **Consistency**: Matches existing UI patterns

**CSS/Styling**:
- [ ] Tailwind classes used (not inline styles)
- [ ] No fixed pixels (use rem)
- [ ] Colors from design system
- [ ] Spacing consistent

---

## ✅ Review Outcomes

### Approve ✅

Code is approved when:
- ✅ All tests passing
- ✅ No architectural concerns
- ✅ Security reviewed
- ✅ Quality standards met
- ✅ Documentation adequate
- ✅ No major issues found

**Comment**:
```markdown
LGTM! ✅

Nice work on [specific thing done well].

Reviewed:
- Architecture ✅
- Code quality ✅
- Tests ✅
- Security ✅
- Performance ✅

Ready to merge.
```

---

### Request Changes 🔄

Request changes for:
- 🔴 Critical bugs
- 🔴 Security vulnerabilities
- 🔴 Architecture violations
- 🔴 Missing tests
- 🔴 Breaking changes

**Comment Template**:
```markdown
Changes requested. Please address:

**Critical** 🔴:
1. [Issue with clear explanation]
2. [Another critical issue]

**Important** 🟡:
1. [Issue that should be fixed]

**Nice to have** 🟢:
1. [Optional improvement]

Happy to review again once addressed.
```

---

### Comment (Suggestions) 💬

Use when code is good but could be better:

```markdown
Looks good overall! A few suggestions:

**Suggestions**:
1. Consider [alternative approach] for [reason]
2. You might want to [improvement]

Not blocking, but would improve the code.
```

---

## 🚨 Common Review Issues

### Issue 1: Thing Object Mishandling
**Problem**: Treating SurrealDB IDs as strings
```typescript
// ❌ Wrong
const companyId = contact.company  // This is a Thing object

// ✅ Correct
const companyId = extractId(contact.company)
```

### Issue 2: Missing "DELETE ME" in Tests
**Problem**: Test data contamination
```typescript
// ❌ Wrong
const testName = "Test Company"

// ✅ Correct
const testName = `DELETE ME - Test Company ${Date.now()}`
```

### Issue 3: Fixed Pixels in UI
**Problem**: Doesn't scale with DPI
```svelte
<!-- ❌ Wrong -->
<div class="h-[32px]">

<!-- ✅ Correct -->
<div class="h-8">  <!-- rem-based -->
```

### Issue 4: No Error Handling
**Problem**: Crashes on edge cases
```typescript
// ❌ Wrong
const result = await invoke('command')

// ✅ Correct
try {
  const result = await invoke('command')
} catch (error) {
  console.error('Failed:', error)
  // Handle appropriately
}
```

### Issue 5: Unwrap in Rust
**Problem**: Panic on errors
```rust
// ❌ Wrong
let value = result.unwrap();

// ✅ Correct
let value = result.map_err(|e| format!("Error: {}", e))?;
```

---

## 📝 Review Comments Best Practices

### Good Comment Structure

```markdown
## [Category]: [Brief summary]

**Current**:
[Show problematic code]

**Issue**:
[Explain why it's a problem]

**Suggested Fix**:
[Show better approach]

**Why**:
[Explain benefit of suggested approach]
```

### Example:

```markdown
## Performance: Database query in loop

**Current**:
```typescript
for (const contact of contacts) {
  const company = await db.query('SELECT * FROM companies WHERE id = $id', { id: contact.company })
}
```

**Issue**:
This creates N+1 queries, one for each contact.

**Suggested Fix**:
```typescript
const companyIds = contacts.map(c => extractId(c.company))
const companies = await db.query('SELECT * FROM companies WHERE id IN $ids', { ids: companyIds })
```

**Why**:
Single query is much faster, especially with many contacts.
```

---

## 🎯 Review Checklist Template

Use this for every review:

### Architecture
- [ ] Follows project patterns
- [ ] No code duplication
- [ ] Appropriate abstraction
- [ ] No architectural violations

### Code Quality
- [ ] Readable and maintainable
- [ ] Proper error handling
- [ ] Type safety
- [ ] No code smells

### Testing
- [ ] Tests exist
- [ ] Tests pass
- [ ] Good coverage
- [ ] "DELETE ME" used

### Security
- [ ] No credentials
- [ ] Input validated
- [ ] Injection prevented
- [ ] Authorization checked

### Performance
- [ ] No obvious bottlenecks
- [ ] Queries optimized
- [ ] Async operations appropriate

### UI/UX (if applicable)
- [ ] DPI scaling works
- [ ] Accessible
- [ ] Consistent styling
- [ ] Good UX

---

## 📊 Review Metrics

Track these over time:

- **Review time**: _[minutes]_
- **Issues found**: _[count by severity]_
- **Review outcome**: Approve / Request Changes / Comment
- **Files reviewed**: _[count]_
- **Lines changed**: _[+XXX / -YYY]_

---

## 💡 Pro Tips

1. **Review in order**: Architecture → Quality → Tests → Security → UI

2. **Be specific**: Point to exact lines, explain why

3. **Suggest, don't demand**: "Consider X" vs "Change to X"

4. **Praise good code**: Acknowledge what's done well

5. **Use checklist**: Don't rely on memory

6. **Test locally**: For complex changes, pull and test

7. **Time-box**: If >1 hour, consider pairing instead

8. **Focus on important**: Don't nitpick every tiny thing

---

## 📚 Related Resources

- [Code Reviewer Specialist](../subagents/subagent-code-reviewer.md) - Detailed guidance
- [Development Workflow](../rules/development-workflow.md) - Project standards
- [Testing Strategy](../context/testing-strategy.md) - Test expectations

---

**Happy reviewing!** 🔍

Remember: Code review is about collaboration, not criticism. Help make the code better while respecting the author's work.
