# Fee Proposal Management System

## Overview
Premium desktop application built with Tauri v2 and Svelte 5 for managing fee proposals, projects, companies, and contacts. Implements emittiv brand design system with SurrealDB integration.

## Tech Stack
- **Frontend**: Svelte 5 (TypeScript, `mount()` API)
- **Desktop**: Tauri v2 (Rust backend)
- **Database**: SurrealDB (WebSocket)
- **Styling**: TailwindCSS + Emittiv palette
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
- **URL**: ws://10.0.1.17:8000
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
- **Contact creation fully working** - All required fields including manual full_name computation
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
git push origin main  # To git.mms.name/martin/fee-prop.git
```

## Next Steps

### Priority 1: Complete CRUD Operations
1. **Projects CRUD** - Apply same pattern as Companies/Contacts
   - Backend: `update_project`, `delete_project` commands  
   - Frontend: Project modal with edit/delete functionality
   - File: `src/lib/components/ProjectModal.svelte` (needs creation)

2. **RFPs/Fee Proposals CRUD** - Apply same pattern
   - Backend: `update_rfp`, `delete_rfp` commands
   - Frontend: RFP modal with edit/delete functionality
   - File: `src/lib/components/RfpModal.svelte` (needs creation)

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

---
**Last Updated**: July 19, 2025  
**Status**: Contact & Company CRUD complete, Projects/RFPs next

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