# Git Workflow Quick Reference

> **Purpose**: Git workflow and commit conventions
> **Use**: Follow these patterns for all git operations

---

## 🌿 Branch Strategy

### Branch Naming

```bash
# Features
feat/contact-search
feat/invoice-export
feat/dark-mode

# Bug fixes
fix/socket-timeout
fix/dpi-scaling-issue
fix/data-corruption

# Refactoring
refactor/database-layer
refactor/component-structure

# Documentation
docs/api-documentation
docs/setup-guide

# Chores
chore/dependency-updates
chore/cleanup-tests
```

### Creating Branches

```bash
# Always branch from main/develop
git checkout main
git pull origin main
git checkout -b feat/your-feature

# For bug fixes from production
git checkout main
git checkout -b fix/bug-description
```

---

## 📝 Conventional Commits

### Format

```
type(scope): short description

Longer description if needed.

- Bullet points for details
- Multiple changes

Closes #issue-number
```

### Types

| Type | When to Use | Example |
|------|-------------|---------|
| `feat` | New feature | `feat(contacts): add search functionality` |
| `fix` | Bug fix | `fix(socket): resolve timeout issue` |
| `refactor` | Code restructuring | `refactor(db): simplify query logic` |
| `test` | Adding/updating tests | `test(e2e): add contact workflow test` |
| `docs` | Documentation | `docs(readme): update setup instructions` |
| `style` | Formatting, no logic change | `style(ui): adjust button padding` |
| `chore` | Maintenance | `chore(deps): update dependencies` |
| `perf` | Performance improvement | `perf(db): optimize contact queries` |

### Scopes

Common scopes in e-fees:

- `contacts`, `companies`, `invoices`, `projects` - Domain features
- `db` - Database changes
- `ui` - User interface
- `api` - Tauri IPC commands
- `e2e` - E2E tests
- `mcp` - MCP server/testing
- `build` - Build system
- `deps` - Dependencies

---

## ✅ Good Commit Examples

```bash
# Feature
git commit -m "feat(contacts): add phone number validation

- Add regex validation for phone format
- Display error message on invalid input
- Add unit tests for validation"

# Bug fix
git commit -m "fix(dpi): correct button sizing at 125% scale

Root cause: Fixed pixels don't scale with OS zoom
Solution: Convert h-[32px] to h-8 (rem-based)

- Update all buttons to use rem-based sizing
- Add regression test at multiple DPI scales

Closes #123"

# Refactor
git commit -m "refactor(db): extract Thing ID helper

- Move extractId to shared utility
- Update all components to use helper
- Remove duplicate implementations"

# Test
git commit -m "test(e2e): add invoice creation workflow

- Test full invoice creation flow
- Verify data persists correctly
- Use DELETE ME pattern for test data"
```

---

## ❌ Bad Commit Examples

```bash
# ❌ Too vague
git commit -m "fix stuff"

# ❌ No type/scope
git commit -m "Added new feature"

# ❌ Multiple unrelated changes
git commit -m "feat: add search + fix bug + update deps + refactor code"

# ❌ Not descriptive
git commit -m "fix(ui): fix"

# ❌ Wrong type
git commit -m "feat(ui): fix button color"  # Should be "fix" or "style"
```

---

## 🔄 Workflow Steps

### 1. Start New Work

```bash
# Update main
git checkout main
git pull origin main

# Create branch
git checkout -b feat/feature-name

# Verify branch
git branch --show-current
```

### 2. Make Changes

```bash
# Check status often
git status

# View changes
git diff

# Stage specific files
git add src/lib/components/NewComponent.svelte
git add src-tauri/src/commands/new_command.rs

# Or stage all
git add .
```

### 3. Commit

```bash
# Commit with message
git commit -m "feat(scope): description"

# Or use editor for longer message
git commit
```

### 4. Push

```bash
# First push (set upstream)
git push -u origin feat/feature-name

# Subsequent pushes
git push
```

### 5. Create PR

```bash
# Using GitHub CLI
gh pr create --title "Feature: Description" --body "Details"

# Or push and create on GitHub web
```

---

## 🔍 Common Operations

### Viewing History

```bash
# Recent commits
git log --oneline -10

# With diffs
git log -p -3

# Graphical
git log --graph --oneline --all

# For specific file
git log --follow path/to/file
```

### Undoing Changes

```bash
# Discard unstaged changes
git checkout -- path/to/file

# Unstage file
git reset HEAD path/to/file

# Amend last commit
git commit --amend

# Undo last commit (keep changes)
git reset --soft HEAD~1

# Undo last commit (discard changes)
git reset --hard HEAD~1
```

### Stashing

```bash
# Save changes
git stash

# Save with message
git stash save "WIP: feature work"

# List stashes
git stash list

# Apply latest
git stash pop

# Apply specific
git stash apply stash@{1}
```

---

## 🔀 Merging & Rebasing

### Updating Branch

```bash
# Merge approach
git checkout feat/feature-name
git merge main

# Rebase approach (cleaner history)
git checkout feat/feature-name
git rebase main

# If conflicts, resolve and:
git add .
git rebase --continue
```

### Squashing Commits

```bash
# Interactive rebase (last 3 commits)
git rebase -i HEAD~3

# In editor, mark commits as:
# pick = keep
# squash = combine with previous
# fixup = like squash but discard message
```

---

## 🚨 Emergency Procedures

### Accidentally Committed to Wrong Branch

```bash
# Save commit hash
git log -1  # Copy the hash

# Reset branch
git reset --hard HEAD~1

# Checkout correct branch
git checkout correct-branch

# Apply commit
git cherry-pick <commit-hash>
```

### Need to Undo Published Commit

```bash
# ⚠️ Only if not shared with team
git revert <commit-hash>
git push

# Or if commit was just pushed
git reset --hard HEAD~1
git push --force-with-lease
```

### Recover Deleted Branch

```bash
# Find commit hash
git reflog

# Recreate branch
git checkout -b recovered-branch <commit-hash>
```

---

## 📊 Pre-Commit Checklist

Before committing:

- [ ] Changes are related (same logical change)
- [ ] All tests pass (`npm test`)
- [ ] No test data left behind (`npm run test:e2e:verify-clean`)
- [ ] Code follows project patterns
- [ ] TypeScript compiles (`npm run check`)
- [ ] Rust compiles without warnings (`cargo check`)
- [ ] Commit message follows convention
- [ ] No console.log/println left in code
- [ ] No credentials in code

---

## 🎯 PR Checklist

Before creating PR:

- [ ] Branch is up to date with main
- [ ] All commits follow convention
- [ ] Tests added for new functionality
- [ ] Documentation updated if needed
- [ ] Self-reviewed code
- [ ] No merge conflicts
- [ ] CI/CD passes (if configured)
- [ ] Ready for review

---

## 📝 PR Description Template

```markdown
## Summary
[Brief description of what this PR does]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Refactoring
- [ ] Documentation

## Changes Made
- [List key changes]
- [Another change]

## Testing
- [ ] Unit tests added/updated
- [ ] E2E tests added/updated
- [ ] Manual testing completed

## Screenshots (if UI changes)
[Add screenshots]

## Checklist
- [ ] Code follows project patterns
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or documented)

## Related Issues
Closes #[issue-number]
```

---

## 💡 Pro Tips

### 1. Atomic Commits
**Do**: One logical change per commit
```bash
git add src/lib/components/Button.svelte
git commit -m "feat(ui): add primary button component"

git add src/lib/components/Button.test.ts
git commit -m "test(ui): add button component tests"
```

**Don't**: Mix unrelated changes
```bash
git add .
git commit -m "feat: add button and fix bug and update deps"
```

### 2. Write Good Messages
**Do**: Explain WHY, not WHAT
```bash
git commit -m "fix(db): add null check for company field

Root cause: contact.company can be null for personal contacts
Solution: Use optional chaining and default to 'Individual'"
```

**Don't**: Just state the obvious
```bash
git commit -m "fix: changed code"
```

### 3. Commit Often
- Commit after each logical step
- Easy to revert if needed
- Better history for debugging

### 4. Use .gitignore
Never commit:
```bash
# Already in .gitignore
.env.local
node_modules/
target/
.claude/settings.local.json
```

---

## 🔗 Related Resources

- [Development Workflow](../rules/development-workflow.md) - Rule 2 (Commit conventions)
- [GitHub Flow](https://guides.github.com/introduction/flow/) - Branching strategy
- [Conventional Commits](https://www.conventionalcommits.org/) - Full spec

---

**Remember**: Good git hygiene makes collaboration easier and debugging faster. When in doubt, commit small and commit often!
