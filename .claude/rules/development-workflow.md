# E-Fees Development Workflow Rules

Project-specific rules only. Workspace-wide standards (commit format, TDD, sub-agent routing, security) live in CLAUDE.md and the global rules - not duplicated here.

## Testing

### Test data safety (MANDATORY)
All test data MUST use the "DELETE ME" prefix. E2E cleanup verification fails the suite if test data remains.

```typescript
// ✅ CORRECT
const testData = {
  name: `DELETE ME - Test Company ${Date.now()}`,
  email: `delete-me-${Date.now()}@example.com`
}
```

### Test before committing
```bash
cargo test -p app --lib                  # Rust unit tests
npm run test:unit                        # Frontend unit tests
cargo test --test integration_*         # If backend changed
npm run test:e2e                         # If critical paths changed (Tauri MCP only)
npm run test:e2e:verify-clean            # No test data left behind
cargo clippy --all-targets --all-features
npm run lint
```

Coverage expectations: bug fixes always get a regression test; new Tauri commands always get an integration test; critical paths get E2E coverage.

## Code Organization

### File placement
| File Type | Location |
|-----------|----------|
| Tauri commands | `src-tauri/src/commands/<module>/` |
| Database queries | `src-tauri/src/db/<module>/` |
| Domain models (shared) | `crates/e-fees-core/src/models/` |
| TypeScript types | `src/lib/types/` |
| Svelte components | `src/lib/components/<module>/` |
| E2E tests | `e2e-mcp/` |
| Standalone API routes | `e-fees-api/src/routes/` |

One component per file. Modules re-export through `mod.rs`.

## Adding a Tauri Command (checklist)
1. Define `#[tauri::command]` in `src-tauri/src/commands/<module>/`
2. Re-export in the module's `mod.rs`
3. Register in the `invoke_handler` in `src-tauri/src/lib.rs`
4. Add TypeScript types in `src/lib/types/`
5. Create frontend wrapper in `src/lib/api/`
6. Integration test; E2E test if critical path
7. If shared logic: put it in `crates/e-fees-core/` and re-export, so e-fees-api can use it too

## Styling

This is a Tauri **desktop app with OS-level DPI scaling** (150-200% on target machines):
- **Use fixed `px` values** in CSS - the OS handles scaling. **Never `rem`** (double-scaling risk on 4K/Retina).
- Use existing semantic classes from `app.css` before writing new ones: `.emittiv-input`, `.emittiv-select`, `.emittiv-btn`, `.emittiv-bulk-bar`, `.emittiv-checkbox`, etc.
- Extract any style pattern repeated 2+ times into a component or CSS class.
- Tailwind was removed 2026-02. Do not add utility-class frameworks.

## Database Queries
- Use indexes for lookups; `LIMIT` result sets; fetch only needed fields.
- No N+1 queries in loops - batch or traverse links (`project_id.name AS project_name`).
- Critical query patterns (type::record, NONE vs NULL, SCHEMAFULL rules) are in CLAUDE.md.

## Frontend Performance
- Debounce search inputs; lazy-load heavy components; don't re-fetch on every render.

## Input Validation
Validate at all three layers: frontend (UX), Tauri command handler (security boundary), database schema (integrity). See `.claude/rules/security-rules.md` if present.

## Branches
`<type>/<description-with-dashes>` - e.g. `feat/add-contact-search`, `fix/mcp-socket-timeout`. Commit format and Co-Authored-By trailer: see CLAUDE.md.

---
**Last Updated**: 2026-06-12 (Fable 5 slim-down from 621 lines; history in git)
