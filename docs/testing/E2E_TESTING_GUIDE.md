# End-to-End Testing Guide

**The E2E testing path for E-Fees is `e2e-mcp/`, using the Tauri MCP server.**
This document used to describe a Playwright/browser-based `e2e/` test suite in
detail (573 lines). That suite is removed as of the 2026-09-15 security pass
(PR-2): browser automation cannot attach to a Tauri desktop app's native
webview, so `playwright.config.ts` had deliberately thrown on load since it
was written, and every `npm run test:e2e*` script pointed at it - meaning
those scripts were guaranteed-red the moment anyone ran them. See
`docs/testing/CRITICAL_DIRECTIVE_TAURI_MCP_ONLY.md` for the full "why
Playwright doesn't work here" explanation.

## Where to actually go

- **`e2e-mcp/README.md`** - the complete, current guide: directory
  structure, quick start, test data safety ("DELETE ME" prefix and cleanup
  utilities), the MCP client API, CI integration, troubleshooting, and a
  before/after conversion example from the old Playwright style to the
  current MCP style.
- **`docs/testing/E2E_TESTING_MCP_IMPLEMENTATION_GUIDE.md`** - the dated
  (2025-08-21) implementation record for how `e2e-mcp/` was built, kept for
  historical context.
- **`CLAUDE.md` § Critical Directives** - the one-line rule every session
  should already have loaded: "ALL E2E testing MUST use the Tauri MCP
  server - NEVER browser-based tools."

## Running the tests

```bash
npm run test:e2e:mcp            # run the suite
npm run test:e2e:safe           # run + cleanup + verify-clean, in one step
npm run test:e2e:cleanup        # remove leftover "DELETE ME" test data
npm run test:e2e:verify-clean   # assert no test data remains
```

There is no `npm run test:e2e` (bare) anymore - it was one of the removed
Playwright scripts. If you're looking for it because an older doc or script
still mentions it, that reference is stale; use `test:e2e:mcp` instead.
