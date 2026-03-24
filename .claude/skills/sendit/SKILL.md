---
name: sendit
description: Use when ready to ship changes for e-fees — runs review→test→bump→tag→CI→verify pipeline autonomously in background. Includes diff-based code review (haiku), auto-fix with test gate, full test suite (cargo test + svelte-check), CI polling, and update.json sync. Pre-requisite: run /smoke-test first (requires app running).
---

# /sendit — E-Fees Ship It Pipeline

Runs the complete e-fees release pipeline autonomously in the background. The main instance stays free while the pipeline executes. A final report is delivered on completion.

## Pre-Requisite: Smoke Tests

**Before running `/sendit`, ensure smoke tests have passed in the current session.**

Smoke tests require the app running with Tauri MCP connected — they can't run inside the autonomous pipeline. Run `/smoke-test` first, then `/sendit`.

```
/smoke-test          ← requires app open, tests 40 checks via Tauri MCP
/sendit              ← autonomous background pipeline
```

The pipeline checks for a smoke test timestamp (`.claude/last-smoke-test`) and warns if tests haven't run recently (>2 hours).

## Arguments

```
/sendit                 — Auto-detect commit type, patch version bump
/sendit fix             — Override commit type to fix:
/sendit feat            — Override commit type to feat:
/sendit minor           — Minor version bump (feat: equivalent)
/sendit major           — Major version bump (breaking change)
/sendit --dry-run       — Show all steps without executing (safe to run anytime)
/sendit --skip-review   — Skip code review (for trivial/docs changes)
/sendit --skip-publish  — Stop after test gate (skip tag + CI + verification)
```

Arguments can be combined: `/sendit fix --skip-review`

## How It Works

1. Parses args from the invocation
2. Spawns a **background Task agent** with the full pipeline
3. Main instance continues working immediately
4. Agent delivers a completion report when done (~25 min for full CI build)

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

4. Tell the user: "Pipeline running in background — I'll notify you when it completes (~25 min)."
5. Continue with other work.

## Pipeline Flow

```
Pre-flight (remotes, tools, smoke test recency)
  → Commit staged changes (if any)
  → Code Review (single pass, diff-only, haiku model)
      → If BLOCK: auto-fix → test suite → pass? commit fix : revert + STOP
      → If WARN/PASS: continue
  → Test Gate (cargo test + npm run check — must pass)
  → Version bump (patch/minor/major)
  → Push (Forgejo + GitHub)
  → Tag → CI poll loop (every 60s, ~20 min)
  → Sync update.json (Forgejo → GitHub)
  → Verify Forgejo release (8 assets)
  → KB observation
  → Final report
```

### Key design decisions

- **Single review pass** — merged cleanit + code review into one haiku-model step. Diff-only, no full file reads. Budget: 8K tokens.
- **Auto-fix with test safety net** — if review finds CRITICAL issues, the agent attempts fixes, then runs the full test suite. Tests fail → revert + stop. No broken code ships.
- **Test gate always runs** — `cargo test` + `npm run check` regardless of whether review was skipped. Pre-existing errors (8 from Proposals.svelte legacy syntax) are tolerated; new errors block.
- **Smoke tests are a pre-step** — they need the app running with Tauri MCP, which the background agent can't provide. Run `/smoke-test` before `/sendit`.

## Key Paths (Hardcoded)

| Item | Value |
|------|-------|
| Version sync script | `scripts/sync-version.cjs` |
| Version source | `package.json` (source of truth) |
| Forgejo remote | `origin` → `forge.mms.name/emittiv/fee-prop` |
| GitHub remote | `github` → `github.com/newillusions/e-fees` |
| GitHub Actions | `gh run list --repo newillusions/e-fees` |
| Base branch | `main` |
| Smoke test timestamp | `.claude/last-smoke-test` |
| Pre-existing type errors | 8 (Proposals.svelte legacy `on:` syntax) |
