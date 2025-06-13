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

**Last Updated**: June 13, 2025  
**Status**: Development Environment Configured, Svelte 5 Integration Complete, Brand Guidelines Applied, Ready for Feature Development  
**Next Steps**: Implement full application features, enhance SplashScreen with actual logo, integrate with live SurrealDB instance