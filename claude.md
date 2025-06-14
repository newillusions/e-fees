# Fee Proposal Management System

## Project Overview
A premium desktop application built with Tauri v2 and Svelte 5 for managing fee proposals, projects, companies, and contacts. The application implements the emittiv brand design system with a sophisticated dark theme, smooth user interactions, and native desktop functionality including SurrealDB integration.

## Technology Stack
- **Frontend**: Svelte 5 with TypeScript (using new `mount()` API)
- **Desktop Framework**: Tauri v2 with Rust backend
- **Database**: SurrealDB integration with WebSocket connection
- **Styling**: TailwindCSS with custom emittiv color palette
- **Build Tool**: Vite with HMR support
- **State Management**: Svelte 5 stores with modern reactivity
- **Router**: svelte-spa-router for client-side navigation
- **Environment**: dotenvy for configuration management

## Design System (Emittiv Brand)

### Colors
```css
--black: #000     /* Primary background */
--darker: #333    /* Secondary background */
--dark: #666      /* Tertiary background */
--light: #999     /* Light text */
--lighter: #ccc   /* Secondary text */
--white: #fff     /* Primary text */
--splash: #f90    /* Orange accent */
```

### Typography
- **Headings**: Ubuntu font family (system-ui fallback)
- **Body**: Montserrat font family (sans-serif fallback)
- **Sizing**: Consistent rem-based scaling (16px base)

### Spacing System
- **Scale**: 0.25rem, 0.5rem, 1rem, 1.5rem, 2rem, 3rem, 4rem
- **Consistent**: All components use standardized spacing

### UI Patterns
- **Gradients**: `linear-gradient(135deg, #000, #333)`
- **Cards**: #333 background, #666 borders, orange hover states
- **Transitions**: `all 0.3s cubic-bezier(0.4, 0, 0.2, 1)`
- **Focus**: Orange ring with offset for accessibility

## Project Structure

```
claude-code-app-1/
├── src/
│   ├── lib/
│   │   ├── components/           # Reusable UI components (Svelte 5)
│   │   │   ├── Card.svelte       # Card with hover effects
│   │   │   ├── Button.svelte     # Button variants (primary/secondary/ghost)
│   │   │   ├── Input.svelte      # Dark theme input component
│   │   │   ├── Layout.svelte     # Main app layout with fixed sidebar
│   │   │   ├── Navigation.svelte # Navigation with keyboard shortcuts
│   │   │   ├── ConnectionStatus.svelte # Pulsing connection indicator
│   │   │   ├── EmptyState.svelte # Consistent empty states
│   │   │   ├── LoadingSkeleton.svelte # Loading animations
│   │   │   └── SplashScreen.svelte # Animated loading screen with logo
│   │   ├── stores/               # Svelte 5 state management
│   │   │   ├── route.ts          # Current route state
│   │   │   ├── connection.ts     # Database connection status
│   │   │   └── data.ts           # Mock data stores
│   │   ├── api.ts                # Tauri command wrappers
│   │   └── utils/                # Utility functions
│   ├── routes/                   # Page components (SPA routes)
│   │   ├── Dashboard.svelte      # Dashboard with stats
│   │   ├── Projects.svelte       # Project management
│   │   ├── Proposals.svelte      # Fee proposal management
│   │   ├── Companies.svelte      # Company directory
│   │   └── Contacts.svelte       # Contact management
│   ├── assets/                   # Static assets
│   │   ├── images/               # Logo and image files
│   │   │   ├── logo-white.svg    # Company logo (white)
│   │   │   └── logo-grey.svg     # Company logo (grey)
│   │   └── index.ts              # Asset exports with ?url imports
│   ├── styles/
│   │   └── app.css               # Global styles and Tailwind + Emittiv
│   ├── types/
│   │   └── index.ts              # TypeScript type definitions
│   ├── App.svelte                # Main app component (Svelte 5)
│   ├── main.ts                   # App entry point (mount() API)
│   └── app.d.ts                  # Ambient declarations + SVG modules
├── src-tauri/
│   ├── src/
│   │   └── main.rs               # Tauri backend
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── public/
│   └── icon.svg                  # App icon
├── index.html                    # HTML entry point
├── package.json                  # NPM dependencies and scripts
├── tailwind.config.js            # Tailwind configuration
├── vite.config.ts                # Vite configuration
├── tsconfig.json                 # TypeScript configuration
└── claude.md                     # This documentation
```

## Key Features Implemented

### 1. Premium Desktop Experience
- **Fixed Sidebar Navigation**: Professional desktop app layout
- **Keyboard Shortcuts**: Cmd/Ctrl + 1-5 for quick navigation
- **Window Configuration**: 1280x800 default, 1024x600 minimum, centered
- **Dark Theme**: Consistent with emittiv brand identity

### 2. Responsive Design
- **Fluid Layouts**: Cards and grids adapt to different screen sizes
- **Consistent Spacing**: Rem-based spacing system ensures scalability
- **Focus Management**: Proper keyboard navigation and focus states

### 3. Interactive Components
- **Card Hover Effects**: Subtle transform and orange border on hover
- **Button Variants**: Primary (orange), secondary (outlined), ghost (minimal)
- **Input Components**: Dark theme with orange focus states
- **Loading States**: Skeleton animations during data loading

### 4. Connection Status
- **Visual Indicator**: Pulsing orange dot when connected
- **Auto-refresh**: Checks connection status every 30 seconds
- **Smooth Transitions**: Color changes with animation

### 5. Data Management
- **Mock Data**: Realistic sample data for all entities
- **Type Safety**: Full TypeScript coverage for all data structures
- **Reactive Stores**: Svelte stores for state management

## Component Library

### Card Component
```svelte
<Card hover={true} padding="p-6">
  Content here
</Card>
```
- **Props**: `hover`, `padding`, `className`
- **Features**: Hover effects, customizable padding

### Button Component
```svelte
<Button variant="primary" size="md">
  Click me
</Button>
```
- **Variants**: `primary`, `secondary`, `ghost`
- **Sizes**: `sm`, `md`, `lg`
- **Features**: Focus states, disabled state, click handlers

### Input Component
```svelte
<Input 
  label="Email" 
  type="email" 
  bind:value={email} 
  required 
/>
```
- **Types**: All standard HTML input types
- **Features**: Labels, validation, error states, focus management

## Navigation & Routing

### Routes
1. **Dashboard** (`Cmd+1`): Overview statistics and recent activity
2. **Projects** (`Cmd+2`): Project management with status tracking
3. **Proposals** (`Cmd+3`): Fee proposal creation and management
4. **Companies** (`Cmd+4`): Company directory with contact counts
5. **Contacts** (`Cmd+5`): Contact management with company associations

### Keyboard Shortcuts
- **Route Navigation**: `Cmd/Ctrl + 1-5`
- **Focus Management**: Tab navigation with proper focus states
- **Accessibility**: ARIA labels and semantic HTML

## Data Models

### Project
```typescript
interface Project {
  id: string;
  name: string;
  description: string;
  status: 'active' | 'completed' | 'on-hold';
  createdAt: Date;
  updatedAt: Date;
}
```

### Proposal
```typescript
interface Proposal {
  id: string;
  title: string;
  client: string;
  amount: number;
  status: 'draft' | 'sent' | 'approved' | 'rejected';
  projectId?: string;
  createdAt: Date;
  dueDate?: Date;
}
```

### Company
```typescript
interface Company {
  id: string;
  name: string;
  industry: string;
  contactCount: number;
  address?: string;
  website?: string;
  createdAt: Date;
}
```

### Contact
```typescript
interface Contact {
  id: string;
  name: string;
  email: string;
  phone?: string;
  position?: string;
  companyId: string;
  company: string;
  createdAt: Date;
}
```

## Development Status & Architecture

### Current Implementation Status ✅
- **Svelte 5 Integration**: Fully updated to use `mount()` API instead of legacy `new App()`
- **Tauri v2 Backend**: Complete Rust backend with SurrealDB integration
- **Database Connection**: WebSocket connection to SurrealDB with authentication
- **Asset Management**: SVG logos properly imported and optimized
- **Splash Screen**: Animated loading screen with company branding
- **Environment Configuration**: External `.env` file for database credentials
- **Development Environment**: WSL2 + Ubuntu 24.04 with all dependencies

### Key Technical Achievements
1. **Resolved Svelte 5 Compatibility**: Updated from legacy component instantiation to modern `mount()` API
2. **Asset Import Resolution**: Fixed SVG imports by removing spaces from filenames and using `?url` suffix
3. **Database Integration**: Complete SurrealDB setup with connection manager and Tauri commands
4. **Rust Compilation**: Successful compilation with all system dependencies installed
5. **Cross-Platform Development**: Working in WSL2 environment with Windows Tauri app

### System Dependencies (Ubuntu 24.04/WSL2)
```bash
# Required packages for Tauri development
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev pkg-config

# Rust installation (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Database Configuration
Current SurrealDB setup:
- **URL**: `ws://10.0.1.17:8000` (local network)
- **Namespace**: `emittiv`
- **Database**: `projects`
- **Authentication**: martin/th38ret3ch
- **Connection**: WebSocket with heartbeat monitoring
- **Fallback**: Mock data when database unavailable

### Svelte 5 Implementation Details

#### Component Instantiation
```typescript
// main.ts - Modern Svelte 5 approach
import { mount } from 'svelte'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})
```

#### Key Svelte 5 Features Used
1. **New `mount()` API**: Replaced legacy `new App()` constructor
2. **Modern Reactivity**: Using `$state()` and `$derived()` where applicable
3. **Improved TypeScript**: Better type inference and checking
4. **Component Composition**: Enhanced prop passing and event handling
5. **Performance**: Better compilation and runtime optimization

#### Asset Management (Svelte 5 Compatible)
```typescript
// src/assets/index.ts
export { default as logoGrey } from './images/logo-grey.svg?url';
export { default as logoWhite } from './images/logo-white.svg?url';
export { default as logo } from './images/logo-white.svg?url';
```

#### Module Declarations
```typescript
// src/app.d.ts - SVG module support
declare module "*.svg" {
  const content: string;
  export default content;
}

declare module "*.svg?url" {
  const content: string;
  export default content;
}
```

## Development Commands

```bash
# Install dependencies
npm install

# Run development server (frontend only)
npm run dev

# Run Tauri development (requires Rust/Cargo)
npm run tauri:dev

# Build for production
npm run tauri:build

# Type checking
npm run check

# Preview production build
npm run preview

# Clear Vite cache (if needed)
rm -rf node_modules/.vite && rm -rf dist
```

## Repository Management

**IMPORTANT**: When discussing repository updates or commits:

1. **Always ask first** before making any commits to the repository
2. **Confirm with user** what should be included in the commit
3. **After committing locally**, always push to the remote Gitea repository at `https://git.mms.name/martin/fee-prop.git`
4. **Verify the push succeeded** and inform the user that changes are now visible in Gitea

**Git Workflow**:
```bash
# 1. Stage changes
git add <files>

# 2. Create commit (only after user approval)
git commit -m "message"

# 3. Push to remote (always required)
git push origin main

# 4. Confirm push succeeded
git status
```

## Tauri Configuration

### Window Settings
- **Size**: 1280x800 (default), 1024x600 (minimum)
- **Position**: Centered on screen
- **Theme**: Dark window frame
- **Resizable**: Yes, with minimum constraints

### Security
- **CSP**: Currently disabled for development
- **API**: Limited to necessary Tauri APIs

## Performance Features

### Loading States
- **Skeleton Loading**: Animated placeholders during data fetching
- **Smooth Transitions**: 300ms transitions with easing
- **Lazy Loading**: Components load on demand

### Optimization
- **Tree Shaking**: Vite optimizes bundle size
- **TypeScript**: Compile-time optimization
- **Svelte Compilation**: Optimal runtime performance

## Accessibility

### Standards Compliance
- **WCAG 2.1**: Proper contrast ratios and focus management
- **Keyboard Navigation**: Full keyboard accessibility
- **Screen Readers**: Semantic HTML and ARIA labels
- **Focus Indicators**: Orange ring with proper offset

### Features
- **Focus Trapping**: Modal focus management
- **Skip Links**: Navigation shortcuts
- **High Contrast**: Emittiv colors meet accessibility standards

## Future Enhancements

### Backend Integration
1. **Database**: SQLite for local storage, PostgreSQL for server
2. **API**: RESTful API with authentication
3. **Sync**: Cloud synchronization capabilities
4. **Backup**: Automated data backup and recovery

### Advanced Features
1. **PDF Generation**: Export proposals to PDF
2. **Email Integration**: Send proposals directly from app
3. **Templates**: Customizable proposal templates
4. **Analytics**: Reporting and business insights
5. **Multi-user**: Team collaboration features
6. **Plugins**: Extensible architecture

### UI/UX Improvements
1. **Search**: Global search across all entities
2. **Filters**: Advanced filtering and sorting
3. **Bulk Actions**: Multi-select operations
4. **Drag & Drop**: File uploads and reordering
5. **Notifications**: In-app notification system
6. **Themes**: Multiple color schemes

## Testing Strategy

### Unit Tests
- **Components**: Svelte Testing Library
- **Utilities**: Jest for pure functions
- **Stores**: Svelte store testing utilities

### Integration Tests
- **User Flows**: End-to-end testing with Playwright
- **API Integration**: Mock API responses
- **Tauri Commands**: Backend integration testing

### Performance Tests
- **Bundle Size**: Analyze and optimize
- **Runtime Performance**: Memory usage and responsiveness
- **Accessibility**: Automated a11y testing

## Troubleshooting & Known Issues

### Development Environment Setup
1. **Svelte 5 Component Error**: If you see "component_api_invalid_new" error:
   - Ensure `main.ts` uses `mount()` instead of `new App()`
   - Check that all components follow Svelte 5 syntax

2. **Asset Loading Issues**: If logos/images don't appear:
   - Remove spaces from asset filenames
   - Use `?url` suffix for asset imports
   - Clear Vite cache: `rm -rf node_modules/.vite`

3. **Database Connection**: If SurrealDB connection fails:
   - Check `.env` file exists with correct credentials
   - Verify SurrealDB server is running on specified URL
   - App will gracefully fall back to mock data

4. **WSL2 Browser Access**: If localhost doesn't work:
   - Try WSL IP address (get with `hostname -I`)
   - Use `--host` flag: `npx vite --host --port 1420`

### Performance Tips
- **First Build**: Initial Rust compilation takes 5-10 minutes
- **Subsequent Builds**: Much faster due to caching
- **Hot Reload**: Vite provides instant updates for frontend changes
- **Memory**: Tauri apps use less memory than Electron equivalents

## Current Working State ✅

### Fully Functional Features
1. **Svelte 5 Application**: Modern component instantiation and reactivity
2. **Splash Screen**: Animated loading with company branding
3. **Desktop Integration**: Native Tauri v2 window with proper theming
4. **Database Layer**: Complete SurrealDB integration with fallbacks
5. **Asset Management**: Optimized SVG logos and favicon support
6. **Development Environment**: WSL2 + Ubuntu 24.04 fully configured
7. **Cross-Platform**: Works in both browser and native desktop app
8. **Brand Consistency**: All emittiv references use proper lowercase styling

### Ready for Next Phase
- **Component Library**: All base components implemented
- **Routing System**: SPA navigation ready
- **Data Layer**: Stores and API wrappers in place
- **Styling System**: Complete Emittiv design system
- **Build Pipeline**: Optimized for development and production

## Deployment

### Desktop Platforms
- **Windows**: `.msi` installer with code signing
- **macOS**: `.dmg` with notarization  
- **Linux**: `.AppImage` and `.deb` packages

### Release Process
1. **Version Bump**: Semantic versioning
2. **Build**: Cross-platform compilation
3. **Test**: Automated testing suite
4. **Sign**: Code signing for security
5. **Distribute**: Auto-updater integration

## Security Considerations

### Data Protection
- **Encryption**: Local data encryption at rest
- **Validation**: Input sanitization and validation
- **Permissions**: Minimal required permissions

### Code Security
- **Dependencies**: Regular security audits
- **CSP**: Content Security Policy in production
- **Tauri**: Secure backend communication

## Brand Consistency

The application maintains strict adherence to the emittiv design system:
- **Colors**: Exact color values from emittiv.com
- **Typography**: Ubuntu and Montserrat font families
- **Spacing**: Consistent rem-based spacing scale
- **Interactions**: Smooth transitions matching web experience
- **Quality**: Premium feel appropriate for professional use

This creates a cohesive brand experience across all emittiv touchpoints while providing the performance and functionality expected from a native desktop application.

---

## Recent Development Session - June 13, 2025

### Database Schema Update Completed ✅

**Major Changes Applied:**
1. **Updated Database Entity Structs** (`src-tauri/src/db/mod.rs`)
   - Replaced old schema with actual SurrealDB schema
   - Added `ProjectNumber`, `TimeStamps`, `Revision` helper structs
   - Updated all field names to match production database

2. **Updated Database Table Names**
   - `"companies"` → `"company"`
   - `"proposals"` → `"rfp"`
   - Updated all Tauri commands and API methods

3. **Updated TypeScript Types** (`src/types/index.ts`)
   - Complete rewrite to match new schema
   - Added proper union types for status/stage enums
   - Maintained backwards compatibility

4. **Updated Frontend Components**
   - All components now use reactive stores instead of mock data
   - Updated field references throughout UI
   - Enhanced error handling and debugging

5. **4K Monitor Configuration** ✅
   - Window positioning: Right half of 4K screen (1920x2160)
   - Automatic positioning on startup
   - Manual repositioning with `Cmd+W` shortcut

### Current Issue: Database Connection Working, No Data Loading

**Status**: Database connects successfully to `ws://10.0.1.17:8000` but queries return 0 results despite data existing in tables.

**Debugging Progress:**
- ✅ SurrealDB connection established (`emittiv` namespace, `projects` database)
- ✅ Authentication successful (root-level credentials)
- ✅ Heartbeat monitoring active
- ✅ All Tauri commands responding
- ❌ All table queries return 0 results

**Debug Tools Added:**
- Store subscription logging
- API call debugging with detailed console output
- "🧪 Test DB Calls" button for manual testing
- "📋 Check Schemas" button for table structure analysis

**SurrealDB MCP Server Configuration:**
```json
{
  "mcpServers": {
    "surrealdb": {
      "command": "mcp-server-surrealdb",
      "args": [],
      "env": {
        "SURREALDB_URL": "ws://10.0.1.17:8000",
        "SURREALDB_NS": "emittiv",
        "SURREALDB_DB": "projects",
        "SURREALDB_USER": "martin",
        "SURREALDB_PASS": "th38ret3ch"
      }
    }
  }
}
```

**Next Session Actions:**
1. Restart Claude Code to activate SurrealDB MCP server
2. Use MCP server to directly query database and identify schema mismatch
3. Fix any field mapping or table structure issues
4. Verify data loads correctly in application

**Files Modified in This Session:**
- `src-tauri/src/db/mod.rs` - Complete schema update
- `src-tauri/src/commands/mod.rs` - Updated command signatures
- `src-tauri/src/lib.rs` - Updated imports and registrations
- `src/types/index.ts` - Complete type definitions rewrite
- `src/lib/api.ts` - Removed mock data fallbacks, added debugging
- `src/lib/stores.ts` - Enhanced with debugging and proper initialization
- `src/routes/*.svelte` - Updated all components to use new schema
- `src-tauri/tauri.conf.json` - 4K monitor window configuration

---

## Recent Updates - June 14, 2025 (Late Evening)

### Database Schema Simplification ✅
**Completed**: Removed unnecessary fields from projects table to simplify the interface

**Changes Made**:
1. **Database Schema** (SurrealDB):
   - Removed `activity`, `package`, and `stage` fields from projects table
   - All existing project data preserved for remaining fields
   - Updated indexes and constraints to reflect simplified schema

2. **Backend Updates**:
   - Updated Rust Project struct in `src-tauri/src/db/mod.rs`
   - Modified search functionality to query relevant fields only
   - Updated all database queries and commands

3. **Frontend Updates**:
   - Updated TypeScript Project interface in `src/types/index.ts`
   - Simplified Projects page layout: Project Name, Area/City/Country, Project Number
   - Removed filter dropdowns for activity and stage
   - Updated search to work with remaining fields only

4. **Documentation Updates**:
   - Updated all schema documentation files
   - Updated query examples and sample data
   - Updated validation rules and error messages

**Result**: Cleaner, more focused project management interface with simplified data model

---

**Last Updated**: June 14, 2025  
**Status**: Database Schema Simplified, All Components Updated  
**Next Steps**: Continue with application development using simplified project structure

### TODO: Documentation & Help System

1. **Help Documentation**: Create a help section or modal that lists all keyboard shortcuts:
   - `Cmd/Ctrl + 1` - Navigate to Dashboard
   - `Cmd/Ctrl + 2` - Navigate to Projects
   - `Cmd/Ctrl + 3` - Navigate to Proposals
   - `Cmd/Ctrl + 4` - Navigate to Companies
   - `Cmd/Ctrl + 5` - Navigate to Contacts
   - `Cmd/Ctrl + W` - Position window for 4K display

2. **User Guide**: Add a user guide accessible from the settings menu that explains:
   - Navigation shortcuts
   - Common workflows
   - Database connection troubleshooting
   - Project numbering system

3. **Onboarding**: Consider adding first-time user tooltips or a quick tour

### IMPORTANT: Database Contains Existing Data
**The SurrealDB database at `ws://10.0.1.17:8000` (namespace: `emittiv`, database: `projects`) already contains data in all tables.** If queries return 0 results, this indicates:
- Authentication/permission issues (even though connection succeeds)
- Incorrect query syntax or scope
- Need to use root-level authentication or different credentials
- The data exists but the current user/scope cannot access it

**DO NOT assume the database is empty when queries return 0 results.**

---

## Database Schema Reference (Added June 14, 2025)

### Database Connection Details
- **URL**: ws://10.0.1.17:8000
- **Namespace**: emittiv
- **Database**: projects
- **User**: martin
- **Password**: Set via SURREALDB_PASS environment variable

### Tables Overview

#### 1. projects table
**Purpose**: Project opportunities emittiv is bidding on  
**ID Format**: `projects:YY_CCCNN` (auto-generated)  
**Type**: SCHEMAFULL

**Fields**:
- `name`: string (required, length > 0)
- `name_short`: string (required)
- `status`: enum (required) - ['Draft', 'RFP', 'Active', 'On Hold', 'Completed', 'Cancelled']
- `area`: string (required)
- `city`: string (required)
- `country`: string (required)
- `folder`: string (required)
- `number`: object (required)
  - `year`: number (required, 20-50)
  - `country`: number (required, dial code)
  - `seq`: number (required, 1-999)
  - `id`: string (computed, format: YY-CCCNN)
- `time`: object (auto-managed)
  - `created_at`: datetime
  - `updated_at`: datetime

**Unique Constraint**: `number.id` must be unique

#### 2. rfp table
**Purpose**: Fee proposals created by emittiv staff  
**ID Format**: `rfp:YY_CCCNN_R` (auto-generated)  
**Type**: SCHEMAFULL

**Fields**:
- `name`: string (required, length > 0)
- `number`: string (required)
- `project_id`: record<projects> (required)
- `company_id`: record<company> (required)
- `contact_id`: record<contacts> (required)
- `status`: enum (default: 'Draft') - ['Draft', 'Active', 'Sent', 'Awarded', 'Lost', 'Cancelled']
- `stage`: enum (default: 'Draft') - ['Draft', 'Prepared', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost']
- `issue_date`: string (required, 6 digits YYMMDD format)
- `activity`: string (optional)
- `package`: string (optional)
- `strap_line`: string (optional)
- `staff_name`: string (optional)
- `staff_email`: string (optional)
- `staff_phone`: string (optional)
- `staff_position`: string (optional)
- `rev`: int (auto-computed from max of revisions array)
- `revisions`: array<object> (default: [])
  - `revision_number`: int (required)
  - `revision_date`: datetime (required)
  - `author_email`: string (required, valid email)
  - `author_name`: string (required)
  - `notes`: string (required)
- `time`: object (auto-managed)

**Unique Constraint**: Combination of `project_id` + `rev` must be unique

#### 3. company table
**Purpose**: Client companies that issue project opportunities  
**ID Format**: `company:ABBREVIATION` (e.g., company:MERAAS)  
**Type**: SCHEMAFULL

**Fields**:
- `name`: string (required, length > 0)
- `name_short`: string (required)
- `abbreviation`: string (required)
- `city`: string (required)
- `country`: string (required)
- `reg_no`: option<string> (optional)
- `tax_no`: option<string> (optional)
- `time`: object (auto-managed)

#### 4. contacts table
**Purpose**: Individual contact persons at client companies  
**ID Format**: `contacts:random_id` (auto-generated)  
**Type**: SCHEMAFULL

**Fields**:
- `first_name`: string (required)
- `last_name`: string (required)
- `email`: string (required, valid email, unique)
- `phone`: string (required, must contain '+', length > 0)
- `position`: string (required)
- `company`: record<company> (required)
- `full_name`: string (auto-computed: first_name + ' ' + last_name)
- `time`: object (auto-managed)

**Unique Constraint**: `email` must be unique across all contacts

#### 5. country table (Reference Data)
**Purpose**: Pre-populated reference table for countries  
**ID Format**: `country:CODE` (e.g., country:AE)  
**Fields**: `name`, `name_formal`, `name_official`, `code`, `code_alt`, `dial_code`, `currency_code`

#### 6. currency table (Reference Data)
**Purpose**: Pre-populated reference table for currencies  
**ID Format**: `currency:CODE` (e.g., currency:USD)  
**Fields**: `code`, `name`

### Business Logic

#### Project Numbering System
- Format: YY-CCCNN (e.g., 24-97101)
- YY: 2-digit year
- CCC: Country dial code (971=UAE, 966=Saudi)
- NN: Sequence number (01-99)
- UAE projects start from 97101, Saudi from 96601
- Sequence resets annually per country

#### RFP Revision Management
- `rev` field is auto-computed from revisions array
- To add a revision: Append object to revisions array
- Required revision fields: revision_number, revision_date, author_email, author_name, notes
- Provides complete audit trail

### Key Relationships
- rfp → projects (many-to-one)
- rfp → company (many-to-one)
- rfp → contacts (many-to-one)
- contacts → company (many-to-one)
- country → currency (many-to-one)

### InDesign Export Mapping
The RFP data exports to JSON with these field mappings:
- "01 Document Name" → rfp.name
- "02 Document Number" → rfp.number
- "03 Document Issue Date" → rfp.issue_date (format as 'dd MMM yyyy')
- "04 Document Revision" → rfp.rev
- "06 Project Number" → project.number.id
- "07 Project Name" → project.name
- "08 Project Name Short" → project.name_short
- "09 Project Location" → project.city
- "10 Project Country" → project.country
- "11 Project Activity" → rfp.activity (moved from project to rfp)
- "12 Project Package" → rfp.package (moved from project to rfp)
- "13 Project Stage" → (removed from schema)
- "21 Client Name" → company.name
- "22 Client Name Short" → company.name_short
- "23 Client City" → company.city
- "24 Client Country" → company.country
- "26 Contact Full Name" → contact.full_name
- "27 Contact Position" → contact.position
- "28 Contact Email" → contact.email
- "29 Contact Phone" → contact.phone
- "99 Strap Line" → rfp.strap_line

### Reference Files Created
The following files contain detailed database information for development:
- `src-tauri/src/db/schema.md` - Complete schema documentation
- `src/types/database.ts` - TypeScript type definitions
- `src-tauri/src/db/entities.rs` - Rust struct definitions
- `src-tauri/src/db/queries.md` - Example SurrealQL queries
- `src-tauri/src/db/validation.md` - Validation rules reference
- `src-tauri/src/db/sample-data.md` - Sample data for testing

These files ensure Claude Code has all necessary database information without requiring direct MCP server access.

### Recent Updates - June 14, 2025 (Evening)

#### Fixed Contacts Page Company Display
**Issue**: Company field showing "[object Object]" instead of company name

**Changes Made**:
1. **Backend** (`src-tauri/src/db/mod.rs`):
   - Updated `get_contacts()` to use `FETCH company` in query
   - Changed `Contact` struct to use `serde_json::Value` for company field
   - This allows handling both Thing ID references and full company objects

2. **Frontend** (`src/routes/Contacts.svelte`):
   - Enhanced `getCompanyName()` function to properly extract company name
   - Now checks for `company.name` property first (full company name)
   - Falls back to `company.name_short` if available
   - Handles various data formats gracefully
   - Updated filter logic to use actual company names

**Technical Details**:
- SurrealDB's `FETCH` directive returns full related records
- The query `SELECT * FROM contacts FETCH company` returns contacts with expanded company data
- Frontend now properly handles the nested company object structure

**Result**: Successfully fixed "[object Object]" display for company names

#### Solution: Client-Side Company Name Lookup
**Approach**: Instead of using SurrealDB's FETCH directive, implemented client-side lookup

**Implementation Details**:
1. **Backend**: Keep simple SELECT queries without FETCH
   - Contacts return company ID as Thing reference
   - Companies loaded separately into companiesStore
   
2. **Frontend** (`src/routes/Contacts.svelte`):
   - Created `getCompanyName()` function that:
     - Accepts company reference (Thing object)
     - Extracts company ID from various formats
     - Looks up company in companiesStore by ID
     - Returns full company name
   - Handles multiple ID formats:
     - Simple string: "company:ABC"
     - Thing object: `{ tb: 'company', id: { String: 'ABC' } }`
     - Fallback to abbreviation if company not found

**Why This Works**:
- Avoids complexity of FETCH queries
- All data loads successfully with standard SELECT
- Company names displayed correctly by cross-referencing stores
- No changes needed to backend data structures

**Key Learning**:
- SurrealDB FETCH can be complex with Rust deserialization
- Client-side joins are simple and effective for small datasets
- This pattern can be reused for other foreign key relationships

### Database Schema Update - June 14, 2025

#### Simplified Projects Table
**Change**: Removed `activity`, `package`, and `stage` fields from projects table

**Rationale**:
- These fields are project-specific attributes that belong at the RFP level
- Activity and package are already present in the RFP table
- Stage was removed entirely from the schema

**Updated Projects Schema**:
- `name`: string (required)
- `name_short`: string (required)
- `status`: enum (required) - ['Draft', 'RFP', 'Active', 'On Hold', 'Completed', 'Cancelled']
- `area`: string (required)
- `city`: string (required)
- `country`: string (required)
- `folder`: string (required)
- `number`: object (required)
- `time`: object (auto-managed)

**Impact**:
- Updated Rust struct in `src-tauri/src/db/mod.rs`
- Updated TypeScript interface in `src/types/index.ts`
- InDesign mapping now pulls activity/package from RFP instead of project

---

## Latest Development Session - June 14, 2025 (End of Day)

### Advanced Filtering System Implementation ✅

#### Enhanced User Interface
**Major Feature Update**: Comprehensive filtering and search system across all entity pages

**Key Features Implemented**:
1. **Reactive Search & Filters**: Real-time filtering as user types or changes selections
2. **Dynamic Filter Options**: Dropdowns populated from actual database data
3. **Consistent UI Pattern**: All pages (Projects, Companies, Contacts, Proposals) follow same design
4. **Smart Data Loading**: Each page loads required data on mount for optimal performance
5. **Alphabetical Sorting**: All filter dropdowns automatically sorted alphabetically
6. **Latest First Ordering**: Records displayed with most recently updated items first

#### Native File Explorer Integration ✅
**Feature**: Project folder paths are now clickable links that open in native file explorer

**Implementation**:
- Added `open_folder_in_explorer` Tauri command for cross-platform support
- Windows: Uses `explorer` command
- macOS: Uses `open` command  
- Linux: Uses `xdg-open` command
- Displays clean folder name while hiding root path for better UX

#### Settings Management System ✅
**Feature**: Complete settings modal with environment file management

**Implementation**:
1. **Settings Modal Component**: Pixel-perfect styling matching emittiv design system
2. **Environment Management**: Read/write `.env` file for database configuration
3. **Validation**: Real-time validation of database URLs and folder paths
4. **Persistence**: Settings automatically saved and loaded across sessions
5. **Native Dialogs**: Folder picker using Tauri's native dialog system

#### Architectural Improvements ✅
**Major Change**: Migrated from derived stores to reactive statements for better performance

**Technical Details**:
1. **From Derived Stores to Reactive Statements**:
   ```typescript
   // Old approach (derived stores)
   export const filteredProjects = derived([projectsStore, searchQuery], ...)
   
   // New approach (reactive statements)
   $: filteredProjects = $projectsStore.filter(project => {
     // Real-time filtering logic
   }).sort((a, b) => new Date(b.time.updated_at) - new Date(a.time.updated_at));
   ```

2. **Benefits of New Architecture**:
   - **Immediate Reactivity**: Filters respond instantly to any change
   - **Better Performance**: No derived store overhead
   - **Real-time Updates**: Database changes still trigger UI updates
   - **Simplified State**: Cleaner component logic
   - **Type Safety**: Better TypeScript inference

#### Page-Specific Enhancements

**Projects Page**:
- Search: Project name, area, city, country
- Filters: Status, Country, City (from actual project data)
- Special: Clickable folder links with native file explorer integration
- Display: Project name, location, project number, folder link

**Companies Page**:
- Search: Company name, short name, abbreviation, city, country
- Filters: Country, City (from actual company data)
- Display: Company name, location, registration/tax numbers

**Contacts Page**:
- Search: Full name, first name, last name, email, phone, position
- Filters: Company (short names), Country, Position (from actual contact data)
- Display: Contact details with company information and direct email links

**Proposals Page**:
- Search: Project name, proposal number, activity, package, staff name
- Filters: Status, Stage, Staff (from actual RFP data)
- Display: Project/proposal details with client and contact information

#### User Experience Improvements
1. **Visual Feedback**: Results counter shows "X of Y items" when filters active
2. **Clear Filters**: One-click button to reset all filters when active
3. **Empty States**: Contextual messages for no data vs no search results
4. **Consistent Spacing**: All elements use exact pixel values for precision
5. **Accessibility**: Full keyboard navigation and screen reader support

#### Development Quality Improvements
1. **Consistent Patterns**: All pages follow identical structure for maintainability
2. **Code Reusability**: Shared functions for common operations
3. **Error Handling**: Graceful degradation when database unavailable
4. **Debugging Tools**: Enhanced logging for development and troubleshooting
5. **Type Safety**: Full TypeScript coverage for all new features

### Files Modified in Latest Session:
- `src-tauri/src/commands/mod.rs` - Added file explorer integration
- `src/lib/components/SettingsModal.svelte` - Complete settings management
- `src/lib/stores/settings.ts` - Settings state management
- `src/routes/Projects.svelte` - Advanced filtering + folder links
- `src/routes/Companies.svelte` - Consistent filtering system
- `src/routes/Contacts.svelte` - Company lookup + filtering
- `src/routes/Proposals.svelte` - Complete filtering overhaul
- `src/lib/stores.ts` - Architecture improvements
- `src-tauri/tauri.conf.json` - Native dialog configuration

### Next Development Phase
**Ready for**: Detail views, form creation/editing, advanced data management features

---

**Last Updated**: June 14, 2025 (End of Day)  
**Status**: Advanced Filtering System Complete, Native Integration Active, Settings Management Operational  
**Architecture**: Reactive filtering system with real-time database updates and optimal performance