# Quick Reference Cards

> **Purpose**: Fast lookup for frequently needed information
> **Reading Time**: 2-5 minutes each
> **When to Use**: Keep these open during active development

---

## 📚 Available Quick References

| Card | Purpose | When to Use | Size |
|------|---------|-------------|------|
| [Commands](commands.md) | Common CLI commands | Every session | 3 KB |
| [Patterns](patterns.md) | Code patterns & examples | When writing code | 5 KB |
| [Git Workflow](git-workflow.md) | Git & commit conventions | Before committing | 4 KB |

**Total**: ~12 KB

---

## 🎯 How to Use

### During Development

**Keep open in separate tab/window**:
1. [Commands](commands.md) - For quick command lookup
2. [Patterns](patterns.md) - When implementing features

### Before Committing

**Review**:
1. [Git Workflow](git-workflow.md) - Ensure proper commit format

### Copy-Paste Friendly

All code examples are designed to be:
- ✅ Copy-paste ready
- ✅ Tested and working
- ✅ Following project conventions

---

## 📖 Quick Reference vs Full Documentation

### Use Quick References When

- ✅ You know what you need, just need syntax
- ✅ Need a quick command
- ✅ Want to copy a pattern
- ✅ Refreshing memory on conventions
- ✅ Time-sensitive (5-10 second lookup)

### Use Full Documentation When

- 📚 Learning new concept
- 📚 Understanding architecture
- 📚 Troubleshooting complex issue
- 📚 Need detailed explanation
- 📚 Planning new feature

---

## 🚀 Quick Start by Task

### "I need to run a command"
→ [Commands](commands.md)

### "I need to implement X"
→ [Patterns](patterns.md) - Find similar pattern

### "I'm about to commit"
→ [Git Workflow](git-workflow.md) - Check conventions

### "Something's broken"
→ [Commands](commands.md#troubleshooting) - Quick fixes

### "I need more detail"
→ [NAVIGATOR](../NAVIGATOR.md) - Route to full docs

---

## 💡 Pro Tips

### 1. Add to Bookmarks
Bookmark these pages in your browser/editor for instant access

### 2. Print & Pin
Print frequently used cards for quick reference

### 3. Customize
Add your own patterns to local copy (don't commit personal shortcuts)

### 4. Shell Aliases
Add frequently used commands as shell aliases:

```bash
# Add to ~/.zshrc or ~/.bashrc
alias efees-dev="npm run tauri:dev"
alias efees-test="npm run test:e2e"
alias efees-clean="npm run test:e2e:cleanup"
```

---

## 🔄 Update Frequency

These cards are updated:
- ✅ When new patterns emerge
- ✅ When commands change
- ✅ When conventions evolve
- ✅ Quarterly review

**Last Updated**: November 6, 2025

---

## 📊 Comparison with Other Docs

| Doc Type | Size | Detail | Use Case |
|----------|------|--------|----------|
| Quick Ref | 3-5 KB | Minimal | Fast lookup |
| Specialist | 15-20 KB | High | Domain work |
| Context | 20-30 KB | Deep | Architecture |
| Prompt | 5-15 KB | Focused | Specific task |

**Quick refs are 80% faster to scan than full docs**

---

## 🎓 Learning Path

**For new developers**:

1. **Week 1**: Memorize command locations
   - Know where to find commands fast
   - Bookmark frequently used

2. **Week 2**: Learn common patterns
   - Study patterns.md examples
   - Use in actual code

3. **Week 3**: Internalize git workflow
   - Follow conventions strictly
   - Review before each commit

4. **Week 4**: Create personal shortcuts
   - Build on top of quick refs
   - Optimize for your workflow

---

## 📚 Related Documentation

### Entry Points
- [NAVIGATOR](../NAVIGATOR.md) - Route to right docs
- [claude.md](../claude.md) - Main project guide

### Full Documentation
- [Specialists](../subagents/) - Domain experts
- [Context](../context/) - Architecture details
- [Prompts](../prompts/) - Task-specific guides

### Workflows
- [Sessions](../sessions/) - Session templates

---

## 🔗 External Quick References

Recommended external quick refs for the stack:

- [Svelte 5 Cheat Sheet](https://svelte-5-preview.vercel.app/)
- [Tauri v2 API Reference](https://v2.tauri.app/)
- [SurrealDB Quick Guide](https://surrealdb.com/docs/surrealql)
- [Tailwind CSS Cheat Sheet](https://tailwindcomponents.com/cheatsheet/)

---

## ✅ Checklist: Using Quick Refs Effectively

- [ ] Bookmarked in browser/editor
- [ ] Know which card has which info
- [ ] Can find command in <10 seconds
- [ ] Copy patterns without modification
- [ ] Check git format before commit
- [ ] Know when to escalate to full docs

---

**Keep it quick, keep it handy, keep coding!** ⚡
