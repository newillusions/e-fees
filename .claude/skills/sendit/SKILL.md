---
name: sendit
description: Use when ready to ship staged changes for e-fees — runs full commit→cleanit→review→bump→tag→CI→verify pipeline autonomously in background. Handles code quality checks, Forgejo/GitHub push, CI polling with timed checkins, and update.json sync.
---

# /sendit — E-Fees Ship It Pipeline

Runs the complete e-fees release pipeline autonomously in the background. The main instance stays free while the pipeline executes. A final report is delivered on completion.

## Arguments

```
/sendit                 — Auto-detect commit type, patch version bump
/sendit fix             — Override commit type to fix:
/sendit feat            — Override commit type to feat:
/sendit minor           — Minor version bump (feat: equivalent)
/sendit major           — Major version bump (breaking change)
/sendit --dry-run       — Show all steps without executing (safe to run anytime)
/sendit --skip-review   — Skip cleanit + code review (for trivial/docs changes)
/sendit --skip-publish  — Stop after merge (skip tag + CI + verification)
```

Arguments can be combined: `/sendit fix --skip-review`

## How It Works

1. Parses args from the invocation
2. Spawns a **background Task agent** with the full pipeline
3. Main instance continues working immediately
4. Agent delivers a completion report when done (15-25 min for full CI build)

## Instructions to Claude

When `/sendit` (or `/sendit <args>`) is invoked:

1. Read `.claude/skills/sendit/references/agent-prompt.md` to get the full pipeline prompt
2. Extract the args from the invocation (e.g. `fix`, `minor`, `--dry-run`)
3. Spawn a background agent:

```
Agent tool:
  description: "E-Fees sendit pipeline"
  subagent_type: "general-purpose"
  model: "sonnet"
  run_in_background: true
  mode: "auto"
  prompt: [agent-prompt.md content] + "\n\n## Args\n" + [parsed args or "none"]
```

4. Tell the user: "Pipeline running in background — I'll notify you when it completes (expect ~20 min for full CI build)."
5. Continue with other work.

## What the Pipeline Does

```
Pre-flight → Commit → Cleanit (review→simplify→guard) → Code Review
    → Version bump → Push (Forgejo + GitHub)
    → Tag → CI poll loop (every 60s)
    → Pull update.json from Forgejo → Push to GitHub → Verify release
    → Report
```

CI checkins are logged at every poll interval and included in the final report so you can see exactly what happened and when.

## Key Paths (Hardcoded)

| Item | Value |
|------|-------|
| Version sync script | `scripts/sync-version.cjs` |
| Version source | `package.json` (source of truth) |
| Forgejo remote | `origin` → `forge.mms.name/emittiv/fee-prop` |
| GitHub remote | `github` → `github.com/newillusions/e-fees` |
| GitHub Actions | `gh run list --repo newillusions/e-fees` |
| Base branch | `main` |
| Merge strategy | squash |
