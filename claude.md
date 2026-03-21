# E-Fees - Digital Fee Proposal Management System

## 🚨 CRITICAL TESTING DIRECTIVE 🚨
**ALL E2E TESTING MUST USE TAURI MCP SERVER - NEVER BROWSER-BASED TOOLS**
See `CRITICAL_DIRECTIVE_TAURI_MCP_ONLY.md` for mandatory requirements.
Browser testing (Playwright/Puppeteer/Selenium) DOES NOT WORK for Tauri apps.

## Overview
Premium desktop application built with Tauri v2 and Svelte 5 for managing fee proposals, projects, companies, and contacts. Implements emittiv brand design system with SurrealDB integration.

## Tech Stack
- **Frontend**: Svelte 5 (TypeScript, `mount()` API)
- **Desktop**: Tauri v2 (Rust backend)
- **Database**: SurrealDB (WebSocket)
- **Styling**: CSS custom properties (design tokens) + `.emittiv-*` classes
- **Router**: svelte-spa-router
- **Build**: Vite with HMR

## Emittiv Design System
```css
--black: #000     /* Primary bg */
--darker: #333    /* Secondary bg */
--dark: #666      /* Tertiary bg */
--light: #999     /* Light text */
--lighter: #ccc   /* Secondary text */
--white: #fff     /* Primary text */
--splash: #f90    /* Orange accent */
```
- **Fonts**: Ubuntu (headings), Montserrat (body)
- **Spacing**: 0.25rem → 4rem scale
- **Transitions**: 300ms cubic-bezier(0.4, 0, 0.2, 1)

## Database Configuration
- **URL**: ws://10.0.23.11:8000
- **Namespace**: emittiv
- **Database**: projects
- **Auth**: martin/[env variable]

## Database Schema

**Complete schema documentation**: See [DATABASE_SCHEMA.md](./DATABASE_SCHEMA.md)

### Key Tables Summary
- **projects**: Project opportunities (48 records) - Format: YY-CCCNN
- **rfp**: Fee proposals (37 records) - Linked to projects/companies/contacts  
- **company**: Client companies (19 records) - UAE, Saudi, international
- **contacts**: Contact persons - Linked to companies
- **country**: Reference data (~250 countries) - For project numbering
- **currency**: Reference data (~180 currencies) - For pricing

### Project Number Format
- **YY-CCCNN**: 25-97105 (2025, UAE=971, sequence=05)
- **Sequence**: Auto-incremented per country/year
- **Countries**: UAE=971, Saudi=966 (from dial_code field)

## Key Features

### Navigation
- **Keyboard Shortcuts**: Cmd/Ctrl + 1-5 for routes
- **Routes**: Dashboard, Projects, Proposals, Companies, Contacts
- **4K Support**: Cmd+W positions window

### UI Components
- **Card**: Hover effects, customizable padding
- **Button**: Primary/secondary/ghost variants
- **Input**: Dark theme with validation
- **Layout**: Fixed sidebar navigation
- **ConnectionStatus**: Pulsing indicator
- **SplashScreen**: Animated loader

### Data Management
- **Reactive Stores**: Svelte 5 state management
- **Real-time Filtering**: Search + dropdowns
- **Client-side Joins**: Company name lookups
- **Optimistic Updates**: UI updates before DB confirmation

### Native Integration
- **File Explorer**: Click project folders to open
- **Settings Modal**: Environment management
- **Window Config**: 1280x800 default, 1024x600 min

## Development

### Commands
```bash
npm install          # Install deps
npm run dev          # Frontend dev
npm run tauri:dev    # Full app dev
npm run tauri:build  # Production build
npm run check        # Type checking
```

### 🚨 CRITICAL: Tailwind Usage Rules
**NEVER use massive inline Tailwind class strings!** Extract repeated patterns into components or CSS classes.

**Forbidden**:
- Class strings > 50 chars repeated across files
- Arbitrary pixel values (`text-[10px]` - use `text-xs`, `text-xxs`)
- Ignoring existing `emittiv-*` classes in `app.css`

**Required**:
- Use Tailwind ONLY for one-off layout tweaks (`flex`, `gap-2`, `ml-1`)
- Extract patterns repeated 2+ times into components or CSS classes
- Use existing `.emittiv-input`, `.emittiv-select`, `.emittiv-btn` classes

**See**: `.claude/rules/development-workflow.md` Rule 19a for complete guidelines.

### System Dependencies (Ubuntu/WSL2)
```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev pkg-config
```

### Svelte 5 Implementation
```typescript
// main.ts
import { mount } from 'svelte'
import App from './App.svelte'

mount(App, { target: document.getElementById('app')! })
```

### Asset Management
```typescript
// SVG imports with ?url suffix
export { default as logo } from './images/logo-white.svg?url';
```

## Current Status ✅

### Completed Features
- **Company CRUD system** with modals ✅
- **Contact CRUD system** with modals ✅ 
- Advanced filtering across all pages
- Native file explorer integration
- Settings management
- Database persistence
- Reactive UI updates
- 4K monitor support

### Architecture Improvements
- Migrated from derived stores to reactive statements
- Client-side joins for foreign keys
- Optimistic UI updates
- Consistent error handling

### Recent Fixes (July 2025)
- **Contact CRUD fully working** ✅ - Create, update, delete with proper ID extraction and form validation
- **Contact modal improvements** ✅ - Fuzzy search company selection, reactive state fixes, 500px width
- **Navigation reordered** ✅ - Dashboard, Projects, Companies, Contacts, Proposals (with Cmd+1-5 shortcuts)
- **Debug logging cleanup** ✅ - Removed console.log/println! statements from contact operations
- **Company ID extraction** - Fixed SurrealDB Thing object handling in frontend
- **Form input issues** - Removed excessive logging causing input blocking
- **Window positioning** - Removed automatic positioning for new environment
- **Database schema documentation** - Complete analysis in DATABASE_STRUCTURE_AND_DEBUGGING.md
- **Test data templates** - Safe creation scripts in TEST_DATA_CREATION.md

## Git Workflow
**IMPORTANT**: Always ask before committing
```bash
git add <files>
git commit -m "message"
git push origin main  # To forge.mms.name/emittiv/fee-prop.git
```

## Release Process
**MANDATORY**: Always use `/release` via a **background haiku agent**. Never run the release pipeline interactively.

```
When user says "/release", "/release patch", "/release minor", "/release major", or "/release X.Y.Z":

1. Spawn a background Task agent:
   - subagent_type: "Bash"
   - model: "haiku"
   - run_in_background: true
   - Provide the full pipeline steps from .claude/commands/release.md

2. Notify user: "Release pipeline running in background. You'll be notified when complete."

3. Continue working on other tasks while the release runs.
```

**Why background**: The CI build takes 15-25 minutes. Running it interactively wastes tokens polling. Haiku handles the scripted git/gh commands and polling loop at minimal cost.

**When to intervene directly**: Only if the background agent reports a build failure. In that case, investigate the failure logs, fix the issue, and re-run the release as a new background agent.

**Key files**: `.claude/commands/release.md` (full pipeline), `scripts/sync-version.cjs` (version sync), `.github/workflows/build-releases.yml` (CI)

## Next Steps

### Priority 1: CRUD Operations - ALL COMPLETE ✅
1. **✅ Contacts CRUD** - COMPLETED
   - Backend: All contact commands implemented with proper ID extraction
   - Frontend: ContactModal with fuzzy search and validation
   - Default sorting by last updated timestamp

2. **✅ Proposals CRUD** - COMPLETED (Verified Nov 2025)
   - Backend: All commands verified (`create_fee`, `get_fees`, `update_fee`, `delete_fee`)
   - Frontend: ProposalModal with delete functionality (line 498-512)
   - Full CRUD operations working as expected

3. **✅ Projects CRUD** - COMPLETED (Verified Nov 2025)
   - Backend: `update_project` (line 1486), `delete_project` commands exist
   - Frontend: ProjectModal.svelte + NewProjectModal.svelte fully functional
   - Status change workflow and folder management integrated

### Priority 2: Enhanced UI Features
3. **Detail views with slide-in panels** - Rich data display
4. **Bulk operations** - Multi-select and batch actions
5. **Advanced filtering** - Date ranges, status combinations

### Priority 3: Business Features  
6. **Project folder integration** - Auto-create/open project directories
7. **InDesign export functionality** - Generate formatted proposals
8. **Help documentation** - In-app user guide

## Troubleshooting

### Common Issues
- **Svelte 5 Error**: Use `mount()` not `new App()`
- **Assets Not Loading**: Remove spaces from filenames, use `?url`
- **DB Returns 0 Results**: Check auth/permissions (data exists)
- **WSL2 Access**: Use `hostname -I` for IP address

### Performance Tips
- First Rust build: 5-10 minutes
- Subsequent builds: Much faster
- HMR for frontend changes
- Tauri uses less memory than Electron

## Standards & Workflow

### Workspace Standards
Follow [WORKSPACE_STANDARDS.md](/Volumes/base/dev/.claude/WORKSPACE_STANDARDS.md) for all code style, security, and workflow rules. Key standards for this project:
- **Styling**: CSS custom properties (design tokens) with semantic `.emittiv-*` classes. TailwindCSS is deprecated workspace-wide.
- **Security**: Credentials in `~/.claude/.credentials.env`, MCP configs use `_FROM` wrapper pattern.
- **Code quality**: Auto-formatted by PostToolUse hook (Prettier, rustfmt).

### Conventional Commits
All commits use the format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`

Every commit must include:
```
Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

### Test-Driven Development (Mandatory — Tier 1 Project)
TDD is required for all coding work:
1. **RED** — Write a failing test that defines expected behavior
2. **GREEN** — Write minimum code to make the test pass
3. **REFACTOR** — Clean up while keeping tests green

Applies to: new features, bug fixes (reproduce first), refactors, API changes.
Test before committing: `cargo test -p app --lib` (Rust), `npm test` (frontend).

### Session Workflow
1. `/lamp-on` — Load KB context, sync pending data, check hub messages
2. Check hub messages for pending requests
3. Do work (observe key findings via `kb_observe`)
4. `/lamp-off` — Save context, update HANDOVER.md, close KB session

---
**Last Updated**: March 15, 2026
**Version**: 0.13.8
**Status**: Production-ready. Auto-updater working. KB integration active.

## Planning & Strategy Documentation
- **[docs/planning/SITREP_2025-12-06.md](./docs/planning/SITREP_2025-12-06.md)** - Comprehensive situation report with architecture analysis, critical issues, and 12-week roadmap
- **[docs/planning/HANDOVER_2025-12-10.md](./docs/planning/HANDOVER_2025-12-10.md)** - Most recent session handover
- **[.claude/HANDOVER.md](./.claude/HANDOVER.md)** - Quick context for KB sessions

## Release & Issues Documentation
- **[RELEASE_PROCESS.md](./RELEASE_PROCESS.md)** - Step-by-step guide for publishing releases
- **[KNOWN_ISSUES.md](./KNOWN_ISSUES.md)** - Comprehensive list of issues and workarounds

## Remaining Scalability Issues (See SitRep for details)
1. **Database Mutex Bottleneck** - `src-tauri/src/db/mod.rs` - Only 1 concurrent DB operation
2. **N+1 Query Problem** - `src/lib/stores.ts:165-173` - Client-side joins with O(n*m) complexity

## Development Notes

### Contact Creation Resolution (July 2025)
**Problem**: "Error: Failed to create contact - API returned null"

**Root Causes Identified**:
1. Database requires ALL fields including `full_name` despite VALUE clause
2. SurrealDB Thing objects need string ID extraction in frontend  
3. Excessive logging blocked form inputs in reactive statements

**Key Files Modified**:
- `src-tauri/src/db/mod.rs:712` - Fixed Contact struct, manual full_name computation
- `src/lib/components/ContactModal.svelte:36` - Company ID extraction helper
- `src/lib/stores.ts:351` - Cleaned contact actions logging
- `src-tauri/src/lib.rs` - Removed automatic window positioning

**Documentation Created**:
- `DATABASE_STRUCTURE_AND_DEBUGGING.md` - Complete schema analysis
- `TEST_DATA_CREATION.md` - Safe test data templates