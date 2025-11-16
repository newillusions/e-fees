# Quick Fix Session Template

> **Session Type**: Quick Fix (<10 minutes)
> **Estimated Time**: 5-10 minutes
> **Token Budget**: 15-20 KB
> **Difficulty**: Low

---

## ⚡ Quick Start

**For urgent, small fixes only**. If it's taking longer than 10 minutes, switch to [Bug Fix Session](bug-fix-session.md).

---

## 📋 Pre-Flight Check (30 seconds)

- [ ] Issue is clearly understood
- [ ] You know which file to change
- [ ] Change is <10 lines of code
- [ ] No tests need to be added/modified
- [ ] Not related to architecture or design

**If any are ❌, use Bug Fix Session instead**

---

## 🎯 Minimal Setup

### Load ONLY

1. **[NAVIGATOR.md](../NAVIGATOR.md)** - Section 2 (BROKEN) - 2 KB
2. **[claude.md](../claude.md)** - Quick Troubleshooting only - 3 KB

**Total**: ~5 KB

**DO NOT load**:
- ❌ Specialist files (unless absolutely necessary)
- ❌ Context documentation
- ❌ Full claude.md

---

## 🔧 Common Quick Fixes

### Fix Type 1: Typo/String Fix

**Examples**:
- Button text wrong
- Error message typo
- Variable name spelling

**Action**:
```bash
# Find the file
grep -r "old text" src/

# Edit in place
# Change the typo
# Save

# Verify
npm run tauri:dev
```

**Commit**:
```bash
git commit -m "fix(ui): correct typo in [component]"
```

---

### Fix Type 2: Import Missing

**Examples**:
- Missing import statement
- Wrong import path

**Action**:
```typescript
// Add missing import
import { needed } from '@/lib/utils'
```

**Verify**:
```bash
npm run check  # TypeScript check
npm run tauri:dev
```

**Commit**:
```bash
git commit -m "fix(imports): add missing import for [module]"
```

---

### Fix Type 3: CSS/Styling Tweak

**Examples**:
- Padding adjustment
- Color fix
- Alignment issue

**Action**:
```svelte
<!-- Change Tailwind class -->
<div class="p-4"> <!-- was p-2 -->
```

**⚠️ DPI Warning**: If using fixed pixels, convert to rem!

**Verify**: Test at 100%, 125%, 150% scale

**Commit**:
```bash
git commit -m "fix(style): adjust padding in [component]"
```

---

### Fix Type 4: Configuration Value

**Examples**:
- Wrong port number
- Timeout too short
- Feature flag

**Action**:
```typescript
// Change config value
const TIMEOUT = 5000  // was 3000
```

**Verify**:
```bash
npm run tauri:dev
# Test the specific feature
```

**Commit**:
```bash
git commit -m "fix(config): increase timeout for [feature]"
```

---

### Fix Type 5: Console Warning

**Examples**:
- Unused variable
- Deprecated API
- Key prop missing

**Action**:
```typescript
// Remove unused variable
// const unused = 'value'  // Delete this line

// Or use it
console.log('Debug:', unused)  // If actually needed
```

**Verify**: Console is clean

**Commit**:
```bash
git commit -m "fix(lint): remove unused variable"
```

---

## ✅ Quick Checklist (1 minute)

- [ ] Change made
- [ ] App runs without errors
- [ ] Specific feature works
- [ ] No new console warnings
- [ ] Change committed

**Total time**: 5-10 minutes

---

## 🚨 When to Stop & Escalate

**Stop immediately if**:

- ❌ Taking longer than 10 minutes
- ❌ Need to understand architecture
- ❌ Breaking other things
- ❌ Need to add tests
- ❌ Involves database changes
- ❌ Requires multiple file changes

**Escalate to**: [Bug Fix Session](bug-fix-session.md) or appropriate specialist

---

## 📝 Minimal Commit Message

```bash
# Type: fix (not feat!)
# Scope: affected area
# Description: one line, present tense

git commit -m "fix(scope): change description"
```

**Examples**:
```bash
git commit -m "fix(ui): correct button text"
git commit -m "fix(config): update default timeout"
git commit -m "fix(import): add missing react import"
git commit -m "fix(style): adjust modal padding"
```

---

## 💡 Speed Tips

1. **Use grep** - Find files fast
   ```bash
   grep -r "search term" src/
   ```

2. **Don't overthink** - If it's simple, just fix it

3. **Manual test only** - No need for full test suite

4. **Skip documentation** - For trivial fixes

5. **Commit and move on** - Don't perfect tiny changes

---

## 🎯 Quick Fix Categories

| Category | Max Time | Example |
|----------|----------|---------|
| Typo | 2 min | "Submit" → "Send" |
| Import | 3 min | Add missing import |
| Style | 5 min | Padding adjustment |
| Config | 3 min | Change timeout value |
| Lint | 2 min | Remove unused var |
| Comment | 1 min | Update TODO |

---

## ⚠️ Common Mistakes

### Mistake 1: "While I'm Here..."
**Problem**: Scope creep from simple fix
**Solution**: Stay focused, create separate issue for improvements

### Mistake 2: Skipping Verification
**Problem**: Assume it works without testing
**Solution**: ALWAYS run `npm run tauri:dev` and test

### Mistake 3: Complex Commit Message
**Problem**: Writing paragraph for 1-line change
**Solution**: One line is enough for quick fixes

---

## 📊 Success Metrics

Quick fix is successful when:

- ✅ Fixed in <10 minutes
- ✅ App still works
- ✅ Specific issue resolved
- ✅ Committed
- ✅ Back to other work

**If any are ❌, you needed a Bug Fix Session instead**

---

**Remember**: Quick fixes are for obvious, simple changes. When in doubt, use the full Bug Fix Session template.

⚡ **Speed is the goal, but correctness comes first!**
