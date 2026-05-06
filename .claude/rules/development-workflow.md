# E-Fees Development Workflow Rules
**Mandatory Development Patterns & Best Practices**

## 🎯 Purpose

These rules ensure consistent, high-quality development practices across the e-fees project. Following these patterns:
- Maintains code quality and consistency
- Prevents common bugs and security issues
- Enables effective collaboration with Claude Code
- Ensures comprehensive testing coverage

---

## 🧠 Sub-Agent Usage

### Rule 1: Use sub-agents when they actually pay off

Direct execution is the default. Reach for a sub-agent when one of these holds:
- The work will produce >500 lines of output (verbose tests, long lookups, large code reviews) and would otherwise burn the main context window.
- Two or more genuinely independent workstreams need to run in parallel and don't share state.
- A red-team / second-opinion pass on a plan or change with multi-system blast radius.

#### Specialist references (use when the criteria above are met):

| Task Type | Sub-Agent | Location |
|-----------|-----------|----------|
| MCP debugging | MCP Specialist | `.claude/subagents/subagent-mcp-specialist.md` |
| Tauri IPC commands | Tauri Developer | `.claude/subagents/subagent-tauri-developer.md` |
| E2E testing | Testing Specialist | `.claude/subagents/subagent-testing-specialist.md` |
| Database queries | Database Specialist | `.claude/subagents/subagent-database-specialist.md` |
| Code reviews | Code Reviewer | `.claude/subagents/subagent-code-reviewer.md` |

#### When NOT to delegate

- Single-file edits or one-line changes — direct work is faster and produces less context churn.
- Sequential steps that share state — handing off mid-flow loses context the spawn prompt has to rebuild.
- Quick lookups (one grep, one read) — direct execution is cheaper than spawning.

Skip the specialists for routine project work; pull them in when the criteria above genuinely apply.

---

## 📝 Commit & Branch Conventions

### Rule 2: Follow Conventional Commits

**Format**: `<type>(<scope>): <description>`

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring (no functionality change)
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

**Examples**:
```bash
# ✅ GOOD
git commit -m "feat(contacts): add phone number validation"
git commit -m "fix(mcp): resolve socket connection timeout"
git commit -m "test(e2e): add company creation workflow test"
git commit -m "docs(readme): update installation instructions"

# ❌ BAD
git commit -m "fixed stuff"
git commit -m "WIP"
git commit -m "update"
```

### Rule 3: Branch Naming

**Format**: `<type>/<description-with-dashes>`

**Examples**:
```bash
# ✅ GOOD
git checkout -b feat/add-contact-search
git checkout -b fix/mcp-socket-timeout
git checkout -b refactor/database-queries
git checkout -b test/invoice-e2e-suite

# ❌ BAD
git checkout -b new-feature
git checkout -b john-work
git checkout -b temp
```

### Rule 4: Commit Frequency

**DO**:
- ✅ Commit after each logical unit of work
- ✅ Commit before starting risky refactoring
- ✅ Commit working code (tests passing)

**DON'T**:
- ❌ Commit non-functional code to main branch
- ❌ Make massive commits with unrelated changes
- ❌ Commit commented-out code

---

## 🧪 Testing Requirements

### Rule 5: Test Before Committing

**MANDATORY**: All commits must pass these checks:

```bash
# 1. Run unit tests
cargo test
npm run test:unit

# 2. Run integration tests (if modified backend)
cargo test --test integration_*

# 3. Run E2E tests (if modified critical paths)
npm run test:e2e

# 4. Verify no test data remains
npm run test:e2e:verify-clean
```

### Rule 6: Test Coverage Requirements

| Code Type | Minimum Coverage | When Required |
|-----------|-----------------|---------------|
| New Rust functions | 80% | Always |
| New TypeScript components | 70% | Always |
| Bug fixes | 100% | Always (regression test) |
| Tauri commands | 100% | Always (integration test) |
| Critical paths | 100% | Always (E2E test) |

### Rule 7: Test Data Safety

**MANDATORY**: All test data MUST use "DELETE ME" prefix.

```typescript
// ✅ CORRECT
const testData = {
  name: `DELETE ME - Test Company ${Date.now()}`,
  email: `delete-me-${Date.now()}@example.com`
}

// ❌ WRONG - Will be rejected in code review
const testData = {
  name: "Test Company",
  email: "test@example.com"
}
```

**Enforcement**:
- Pre-commit hook checks for test files without "DELETE ME"
- E2E tests fail if cleanup verification detects test data
- Code review blocks PRs with unsafe test patterns

---

## 🏗️ Code Organization

### Rule 8: File Placement

| File Type | Location | Example |
|-----------|----------|---------|
| Tauri commands | `src-tauri/src/commands/<module>/` | `commands/contacts/create.rs` |
| Database queries | `src-tauri/src/db/<module>/` | `db/contacts/queries.rs` |
| Rust models | `src-tauri/src/models/` | `models/contact.rs` |
| TypeScript types | `src/lib/types/` | `types/contact.ts` |
| Svelte components | `src/lib/components/<module>/` | `components/contacts/ContactForm.svelte` |
| E2E tests | `e2e-mcp/src/tests/` | `tests/contacts.test.ts` |
| MCP tools | `tauri-plugin-mcp/mcp-server-ts/src/tools.ts` | Tool definitions |

### Rule 9: One Component Per File

**DO**:
```
✅ ContactList.svelte
✅ ContactForm.svelte
✅ ContactCard.svelte
```

**DON'T**:
```
❌ Contacts.svelte (contains List, Form, and Card)
```

### Rule 10: Module Boundaries

**Each module should have clear boundaries:**

```rust
// src-tauri/src/commands/contacts/mod.rs
mod create;
mod read;
mod update;
mod delete;

pub use create::create_contact;
pub use read::{get_contact, get_all_contacts};
pub use update::update_contact;
pub use delete::delete_contact;
```

---

## 🔄 Development Workflow

### Rule 11: Feature Development Process

**Standard workflow for new features:**

```bash
# 1. Create feature branch
git checkout -b feat/your-feature

# 2. Implement incrementally
# - Write failing test first (TDD)
# - Implement feature
# - Make tests pass
# - Refactor

# 3. Run full test suite
npm test

# 4. Create pull request
# - Include description
# - Link related issues
# - Add screenshots if UI change

# 5. Code review required
# - Use Code Reviewer sub-agent
# - Address feedback
# - Re-run tests

# 6. Merge only when:
# - All tests pass
# - Code review approved
# - No merge conflicts
```

### Rule 12: Adding Tauri Commands

**Follow this exact sequence:**

```bash
# 1. Define Rust command
# File: src-tauri/src/commands/example/new_command.rs
#[tauri::command]
pub async fn new_command(param: String) -> Result<String, String> {
    // Implementation
}

# 2. Register in module
# File: src-tauri/src/commands/example/mod.rs
pub use new_command::new_command;

# 3. Register in main.rs
# File: src-tauri/src/main.rs
.invoke_handler(tauri::generate_handler![
    commands::example::new_command,
    // ... other commands
])

# 4. Add TypeScript types
# File: src/lib/types/example.ts
export interface NewCommandParams {
  param: string
}

# 5. Create frontend wrapper
# File: src/lib/api/example.ts
export async function newCommand(params: NewCommandParams) {
  return await invoke('new_command', params)
}

# 6. Add integration test
# File: src-tauri/tests/integration_example.rs

# 7. Add E2E test (if critical path)
# File: e2e-mcp/src/tests/example.test.ts

# 8. Add MCP tool (if needed for testing)
# File: tauri-plugin-mcp/mcp-server-ts/src/tools.ts
```

**REFERENCE**: See `.claude/prompts/tauri-development.md` for detailed guidance.

---

## 🐛 Bug Fix Workflow

### Rule 13: Bug Fix Process

```bash
# 1. Reproduce the bug
# - Create a failing test that demonstrates the bug
# - Document steps to reproduce

# 2. Create bug fix branch
git checkout -b fix/bug-description

# 3. Fix the bug
# - Make minimal changes
# - Ensure test now passes

# 4. Add regression test
# - E2E test if user-facing
# - Unit/integration test for internal logic

# 5. Verify no side effects
# - Run full test suite
# - Check related functionality

# 6. Document the fix
# - Update changelog
# - Add code comments if complex
# - Link to issue in commit message
```

---

## 📚 Documentation Requirements

### Rule 14: Code Documentation

**Rust Documentation**:
```rust
/// Creates a new contact in the database.
///
/// # Arguments
/// * `contact` - The contact data to create
/// * `db` - Database state
///
/// # Returns
/// * `Ok(Contact)` - Successfully created contact with ID
/// * `Err(String)` - Error message if creation failed
///
/// # Examples
/// ```rust
/// let contact = Contact { name: "John".to_string(), .. };
/// let result = create_contact(contact, db).await?;
/// ```
#[tauri::command]
pub async fn create_contact(
    contact: Contact,
    db: State<'_, Database>
) -> Result<Contact, String> {
    // Implementation
}
```

**TypeScript Documentation**:
```typescript
/**
 * Creates a new contact via Tauri IPC.
 * 
 * @param contact - Contact data to create
 * @returns Promise resolving to created contact with ID
 * @throws Error if creation fails or validation errors occur
 * 
 * @example
 * const contact = { name: 'John Doe', email: 'john@example.com' }
 * const created = await createContact(contact)
 */
export async function createContact(contact: ContactInput): Promise<Contact> {
  return await invoke('create_contact', { contact })
}
```

### Rule 15: README Updates

**Update README.md when**:
- Adding new features
- Changing setup process
- Modifying dependencies
- Updating environment requirements

---

## 🔍 Code Review Requirements

### Rule 16: Self-Review Before Requesting

**Before requesting review**:
```bash
# 1. Use Code Reviewer sub-agent
# Reference: .claude/subagents/subagent-code-reviewer.md

# 2. Check for:
✅ All tests passing
✅ No commented-out code
✅ No debug statements (console.log, println!)
✅ No hardcoded values
✅ Proper error handling
✅ Documentation complete
✅ "DELETE ME" in all test data

# 3. Run linters
cargo clippy --all-targets --all-features
npm run lint
```

### Rule 17: Review Checklist

**Reviewer must verify**:
- [ ] Follows conventional commits
- [ ] Tests cover new code
- [ ] No security vulnerabilities
- [ ] Performance considerations addressed
- [ ] Documentation updated
- [ ] Breaking changes noted
- [ ] "DELETE ME" pattern used in tests

---

## ⚡ Performance Considerations

### Rule 18: Database Query Optimization

**DO**:
- ✅ Use indexes for lookups
- ✅ Limit result sets with `LIMIT`
- ✅ Fetch only needed fields
- ✅ Batch operations when possible

**DON'T**:
- ❌ Use `SELECT *` unnecessarily
- ❌ Make N+1 queries in loops
- ❌ Fetch entire tables without pagination

**REFERENCE**: See `.claude/prompts/database-patterns.md`

### Rule 19: Frontend Performance

**DO**:
- ✅ Lazy load components
- ✅ Debounce search inputs
- ✅ Virtualize long lists
- ✅ Cache API responses

**DON'T**:
- ❌ Re-fetch on every render
- ❌ Load all data upfront
- ❌ Block UI with long operations

---

## 🎨 Styling & CSS Architecture

### Rule 19a: Tailwind Usage and Component Discipline

**CRITICAL**: Tailwind must NEVER be used with massive inline class strings. Enforce strict component discipline.

#### ⚠️ DESKTOP APP REQUIREMENT: Fixed Pixel Values

**This is a Tauri desktop app with OS-level scaling (typically 150% on target machines).**

- ✅ **USE fixed `px` values** in CSS classes - OS handles scaling automatically
- ❌ **DO NOT use `rem` values** - can cause double-scaling or inconsistent behavior
- ✅ All `emittiv-*` classes use fixed px (e.g., `font-size: 12px`, `padding: 6px 12px`)
- ✅ Tailwind config defines sizes with px (e.g., `'xxs': ['10px', { lineHeight: '14px' }]`)

**Why this matters**: Desktop apps rely on OS-level DPI scaling. Using rem values designed for responsive web can cause incorrect scaling on 4K/Retina displays with 150-200% system scaling.

#### FORBIDDEN ❌

**Never do these:**
- Class strings longer than 50 characters repeated across files
- Copy-pasting utility combinations multiple times
- Arbitrary pixel values like `text-[10px]` (use semantic sizes: `text-xs`, `text-xxs`)
- Ignoring existing CSS classes in `app.css` (`emittiv-input`, `emittiv-select`, `emittiv-btn`)

**Example of what NOT to do:**
```svelte
<!-- ❌ BAD - 100+ char string repeated 8 times -->
<input class="px-1.5 py-0.5 bg-transparent border border-transparent hover:border-emittiv-dark rounded text-emittiv-white text-xs focus:outline-none focus:border-emittiv-splash focus:bg-emittiv-black" />
```

#### REQUIRED ✅

**Component Extraction Rules:**

1. **Tailwind utilities ONLY for one-off layout/spacing tweaks**
   - ✅ `flex`, `gap-2`, `ml-1`, `mt-4` - simple layout utilities
   - ❌ Long combinations of states, colors, borders, padding

2. **Extract ANY pattern that repeats 2+ times into:**
   - Component files (`InlineInput.svelte`, `IconButton.svelte`, `PanelCard.svelte`)
   - CSS classes with `@apply` in `app.css`
   - Existing `emittiv-*` classes

3. **Use semantic text sizes** (never arbitrary values)
   - ✅ `text-xs` (12px), `text-xxs` (10px), `text-sm` (14px)
   - ❌ `text-[10px]`, `text-[13px]`
   - Add custom sizes to `tailwind.config.js` if needed

4. **Components must have clean, readable markup**
   - Styles should be in components or CSS files
   - Markup should focus on structure, not styling details

**Example of proper usage:**
```svelte
<!-- ✅ GOOD - Extract to component or CSS class -->
<InlineInput bind:value={name} className="w-full" />
<!-- or -->
<input class="emittiv-inline-input w-full" bind:value={name} />
```

#### Existing Infrastructure

**USE these existing CSS classes from `app.css`:**
- `.emittiv-input` (lines 395-430) - MUST use for form inputs
- `.emittiv-select` (lines 432-465) - MUST use for dropdowns
- `.emittiv-btn` (lines 561-610) - MUST use for buttons
- CSS variables for Emittiv design system (colors, spacing)

#### Code Review Checklist

**Before committing, check for:**
- [ ] Class strings longer than 50 chars → extract to component
- [ ] Same class combo used 2+ times → extract to CSS class
- [ ] Arbitrary pixel values → use Tailwind scale or add to config
- [ ] Ignored existing CSS classes → use them instead
- [ ] Component markup is clean and readable

**Rationale**: Frontend specialist audit (2026-01-27) found massive technical debt in pricing module:
- 27× `text-[10px]` arbitrary values
- 8× repeated 100+ character class strings
- 50+ inline inputs ignoring `emittiv-input` class
- 400+ lines of unnecessary repetition

**Decision**: Keep Tailwind but enforce proper component extraction. This approach provides a path to pure-CSS if needed later.

---

## 🔒 Security Practices

### Rule 20: Input Validation

**ALWAYS validate inputs at**:
1. Frontend (user experience)
2. Tauri command handler (security boundary)
3. Database schema (data integrity)

**REFERENCE**: See `.claude/rules/security-rules.md` for comprehensive security guidelines.

---

## 🚀 Deployment Workflow

### Rule 21: Pre-Deployment Checklist

**Before deploying**:
```bash
# 1. All tests pass
npm test

# 2. No test data in production DB
npm run test:e2e:verify-clean

# 3. Production build works
npm run tauri:build

# 4. Manual smoke test
# - Start production build
# - Test critical paths
# - Check error handling

# 5. Version bump
npm version patch|minor|major

# 6. Create release tag
git tag v1.2.3
git push origin v1.2.3
```

---

## 📊 Monitoring Development

### Rule 22: Track Development Metrics

**Use conversation files to track**:
- Feature completion status
- Technical debt items
- Known issues
- Performance bottlenecks

**Location**: `.claude/conversations/`

---

## 🔗 Related Documentation

- **Security Rules**: `.claude/rules/security-rules.md`
- **Performance Rules**: `.claude/rules/performance-rules.md`
- **Testing Strategy**: `.claude/context/testing-strategy.md`
- **MCP Architecture**: `.claude/context/mcp-architecture.md`

---

## ⚠️ Enforcement

These rules are enforced through:
1. **Pre-commit hooks** - Automated checks before commits
2. **Code review process** - Manual verification
3. **CI/CD pipeline** - Automated testing and linting
4. **Sub-agent delegation** - Claude Code workflow compliance

**Violations may result in**:
- Pull request rejection
- Required rework
- Additional code review rounds

---

**Last Updated**: October 26, 2025  
**Version**: 1.0  
**Next Review**: Monthly or after major project changes
