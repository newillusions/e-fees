# Fee Proposal Management System

A premium desktop application built with Tauri v2 and Svelte 5 for managing fee proposals, projects, companies, and contacts. Implements the emittiv brand design system with SurrealDB integration for robust data management.

![Application Screenshot](docs/images/dashboard.png)

## 🚀 Features

### Core Functionality
- **Project Management**: Complete CRUD operations with automatic numbering system
- **Company Database**: Manage client organizations with detailed contact information
- **Contact Management**: Individual contacts linked to companies with full profiles
- **RFP Workflow**: Request for Proposal creation with revision tracking
- **Advanced Search**: Real-time filtering across all data types
- **File Integration**: Native file explorer integration for project folders

### Technical Highlights
- **Native Desktop App**: Built with Tauri v2 for optimal performance
- **Modern UI**: Svelte 5 with reactive state management and smooth animations
- **Real-time Database**: SurrealDB with WebSocket connectivity
- **Responsive Design**: Optimized for both standard and 4K displays
- **Cross-platform**: Windows, macOS, and Linux support
- **Type Safety**: Full TypeScript implementation with comprehensive type definitions

## 📋 Table of Contents

- [Quick Start](#-quick-start)
- [System Requirements](#-system-requirements)
- [Installation](#-installation)
- [Configuration](#-configuration)
- [Development](#-development)
- [Database Setup](#-database-setup)
- [Project Structure](#-project-structure)
- [API Documentation](#-api-documentation)
- [Testing](#-testing)
- [Deployment](#-deployment)
- [Troubleshooting](#-troubleshooting)
- [Contributing](#-contributing)

## ⚡ Quick Start

```bash
# 1. Clone the repository
git clone https://git.mms.name/martin/fee-prop.git
cd fee-prop

# 2. Install dependencies
npm install

# 3. Set up environment variables
cp .env.example .env
# Edit .env with your SurrealDB credentials

# 4. Start development server
npm run tauri:dev
```

## 💻 System Requirements

### Minimum Requirements
- **OS**: Windows 10/11, macOS 10.15+, or Linux (Ubuntu 18.04+)
- **RAM**: 4GB minimum, 8GB recommended
- **Storage**: 500MB free space
- **Network**: Internet connection for database access

### Development Requirements
- **Node.js**: v18.0.0 or higher
- **Rust**: Latest stable version (1.70+)
- **SurrealDB**: v1.0.0 or higher
- **Git**: For version control

### Linux Dependencies (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget \
  libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev pkg-config
```

## 🔧 Installation

### 1. Prerequisites

#### Install Node.js
```bash
# Using Node Version Manager (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### Install SurrealDB
```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://install.surrealdb.com | sh

# Windows
iwr https://install.surrealdb.com -useb | iex
```

### 2. Project Setup

```bash
# Clone repository
git clone https://git.mms.name/martin/fee-prop.git
cd fee-prop

# Install Node.js dependencies
npm install

# Install Rust dependencies (automatic on first build)
cd src-tauri
cargo check
cd ..
```

### 3. Environment Configuration

Create `.env` file in the project root:

```env
# Database Configuration
SURREALDB_URL=ws://10.0.1.17:8000
SURREALDB_NS=emittiv
SURREALDB_DB=projects
SURREALDB_USER=martin
SURREALDB_PASS=your_password_here

# Application Settings
PROJECT_FOLDER_PATH=E:\emittiv\emittiv\01 Projects\01 RFPs

# Staff Information (for RFP generation)
STAFF_NAME=Martin Smith
STAFF_EMAIL=martin@emittiv.com
STAFF_PHONE=+971-50-123-4567
STAFF_POSITION=Principal Architect
```

## ⚙️ Configuration

### Database Connection

The application supports multiple SurrealDB connection methods:

#### WebSocket Connection (Recommended)
```env
SURREALDB_URL=ws://10.0.1.17:8000
```

#### HTTP Connection (Fallback)
```env
SURREALDB_URL=http://10.0.1.17:8000
```

### Authentication Levels

The application attempts authentication in this order:
1. **Database Level**: Direct database user authentication
2. **Namespace Level**: Namespace-level user authentication  
3. **Root Level**: System administrator authentication

### File System Paths

Configure project template folders:
```env
PROJECT_FOLDER_PATH=E:\emittiv\emittiv\01 Projects\01 RFPs\_yy-cccnn Project Name
```

## 🛠️ Development

### Development Commands

```bash
# Frontend development server (with HMR)
npm run dev

# Full application development (recommended)
npm run tauri:dev

# Type checking
npm run check

# Build production version
npm run tauri:build

# Test database connection
npm run test:db
```

### Development Workflow

1. **Frontend Changes**: Use `npm run dev` for rapid iteration
2. **Backend Changes**: Use `npm run tauri:dev` for Rust compilation
3. **Database Schema**: Modify `DATABASE_SCHEMA.md` and update Rust types
4. **Testing**: Run `npm run check` before committing

### Hot Module Replacement

The development server supports HMR for:
- Svelte components
- TypeScript files
- CSS styles
- Static assets

Rust changes require a full restart of the development server.

## 🗄️ Database Setup

### SurrealDB Installation

```bash
# Start SurrealDB server
surreal start --log trace --user root --pass root memory

# Or with file storage
surreal start --log trace --user root --pass root file://database.db

# Or with specific binding
surreal start --bind 0.0.0.0:8000 --user root --pass root memory
```

### Database Schema

The application uses the following schema structure:

#### Projects Table
```sql
DEFINE TABLE projects SCHEMAFULL;
DEFINE FIELD name ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD name_short ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD number ON projects TYPE object;
DEFINE FIELD number.year ON projects TYPE int ASSERT $value >= 20 AND $value <= 50;
DEFINE FIELD number.country ON projects TYPE int ASSERT $value > 0;
DEFINE FIELD number.seq ON projects TYPE int ASSERT $value >= 1 AND $value <= 99;
DEFINE FIELD number.id ON projects TYPE string;
```

#### Countries Reference Data
```sql
DEFINE TABLE country SCHEMAFULL;
DEFINE FIELD name ON country TYPE string;
DEFINE FIELD dial_code ON country TYPE int;
```

### Data Migration

If migrating from existing systems:

```bash
# Export existing data
surreal export --conn http://localhost:8000 --user root --pass root --ns emittiv --db projects backup.sql

# Import data
surreal import --conn http://localhost:8000 --user root --pass root --ns emittiv --db projects backup.sql
```

## 📁 Project Structure

```
fee-prop/
├── src/                          # Frontend Svelte application
│   ├── lib/
│   │   ├── components/           # Reusable UI components
│   │   │   ├── Card.svelte      # Base card component
│   │   │   ├── Button.svelte    # Styled button component
│   │   │   ├── Input.svelte     # Form input component
│   │   │   ├── NewProjectModal.svelte  # Project creation modal
│   │   │   └── ConnectionStatus.svelte # Database status indicator
│   │   ├── stores/              # Svelte stores for state management
│   │   │   └── data.ts         # Reactive data stores
│   │   ├── api.ts              # API client for backend communication
│   │   └── utils/              # Utility functions
│   ├── routes/                 # Application pages
│   │   ├── Dashboard.svelte    # Main dashboard
│   │   ├── Projects.svelte     # Projects management
│   │   ├── Companies.svelte    # Companies management
│   │   ├── Contacts.svelte     # Contacts management
│   │   └── Proposals.svelte    # RFP management
│   ├── styles/                 # Global styles and themes
│   │   └── app.css            # Main stylesheet with design system
│   ├── types/                  # TypeScript type definitions
│   │   └── index.ts           # Core type definitions
│   ├── App.svelte             # Root application component
│   └── main.ts                # Application entry point
├── src-tauri/                  # Backend Rust application
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   │   └── mod.rs         # Command implementations
│   │   ├── db/                # Database operations
│   │   │   ├── mod.rs         # Database manager and operations
│   │   │   └── entities.rs    # Database entity definitions
│   │   ├── lib.rs             # Tauri application setup
│   │   └── main.rs            # Application entry point
│   ├── Cargo.toml             # Rust dependencies
│   ├── tauri.conf.json        # Tauri configuration
│   └── build.rs              # Build script
├── static/                     # Static assets
├── docs/                       # Documentation
│   ├── DATABASE_SCHEMA.md     # Complete database schema
│   └── API.md                 # API documentation
├── .env.example               # Environment variables template
├── package.json               # Node.js dependencies and scripts
├── tailwind.config.js         # TailwindCSS configuration
├── tsconfig.json              # TypeScript configuration
├── vite.config.ts             # Vite build configuration
├── CLAUDE.md                  # Project documentation and guidelines
└── README.md                  # This file
```

## 📚 API Documentation

### Frontend API Client

The `ApiClient` class provides type-safe access to all backend operations:

```typescript
import { ApiClient } from '$lib/api';

// Get all projects
const projects = await ApiClient.getProjects();

// Create a new company
const company = await ApiClient.createCompany({
  name: 'Conrad Hilton Hotels',
  name_short: 'Conrad Etihad',
  abbreviation: 'CHE',
  city: 'Dubai',
  country: 'United Arab Emirates'
});

// Generate project number
const projectNumber = await ApiClient.generateNextProjectNumber('United Arab Emirates');
```

### Backend Commands

Tauri commands are defined in `src-tauri/src/commands/mod.rs`:

```rust
// Database operations
get_projects() -> Vec<Project>
create_project(project: Project) -> Project
update_company(id: String, company: CompanyUpdate) -> Company

// Project numbering
generate_next_project_number(country_name: String, year: Option<u8>) -> String
validate_project_number(project_number: String) -> bool

// File operations
open_folder_in_explorer(folder_path: String) -> String
select_folder() -> Option<String>
```

For complete API documentation, see [docs/API.md](docs/API.md).

## 🧪 Testing

### Unit Tests

```bash
# Run Rust tests
cd src-tauri
cargo test

# Run frontend tests
npm run test
```

### Integration Tests

```bash
# Test database connectivity
npm run test:db

# Test complete workflow
npm run test:e2e
```

### Manual Testing Checklist

#### Project Creation
- [ ] Generate unique project numbers
- [ ] Create project folders
- [ ] Copy template files
- [ ] Update database records

#### Data Management
- [ ] CRUD operations for all entities
- [ ] Search and filtering
- [ ] Data validation
- [ ] Error handling

#### UI/UX
- [ ] Responsive design
- [ ] Keyboard shortcuts
- [ ] Modal interactions
- [ ] Real-time updates

## 🚀 Deployment

### Building for Production

```bash
# Build optimized production bundle
npm run tauri:build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`:

- **Windows**: `.msi` installer
- **macOS**: `.dmg` disk image
- **Linux**: `.deb` package and `.tar.gz` archive

### Distribution

#### Windows
```bash
# Sign the executable (optional)
signtool sign /f certificate.pfx /p password target/release/fee-prop.exe

# Create installer
npm run tauri:build
```

#### macOS
```bash
# Code signing and notarization
npm run tauri:build -- --target universal-apple-darwin
```

#### Linux
```bash
# Build for multiple distributions
npm run tauri:build -- --target x86_64-unknown-linux-gnu
```

### Environment-Specific Builds

#### Production Environment
```env
SURREALDB_URL=wss://production-db.example.com
SURREALDB_NS=production
SURREALDB_DB=projects
```

#### Staging Environment
```env
SURREALDB_URL=ws://staging-db.example.com
SURREALDB_NS=staging
SURREALDB_DB=projects
```

## 🔍 Troubleshooting

### Common Issues

#### Database Connection Failures

**Problem**: "No such host is known" error
```
Solution: Check network connectivity and database server status
- Verify SurrealDB is running: `surreal version`
- Test connectivity: `telnet 10.0.1.17 8000`
- Check firewall settings
```

**Problem**: Authentication failed
```
Solution: Verify credentials in .env file
- Check SURREALDB_USER and SURREALDB_PASS
- Ensure user has proper permissions
- Try different authentication levels
```

#### Build Issues

**Problem**: Rust compilation errors
```bash
# Clear Rust cache
cd src-tauri
cargo clean
cargo update

# Reinstall dependencies
cd ..
rm -rf node_modules
npm install
```

**Problem**: Node.js version conflicts
```bash
# Use correct Node.js version
nvm use 18

# Clear npm cache
npm cache clean --force
```

#### Runtime Issues

**Problem**: Application won't start
```
1. Check console for JavaScript errors
2. Verify environment variables are set
3. Ensure database is accessible
4. Check file permissions for project folders
```

**Problem**: Features not working
```
1. Check database connection status
2. Verify user permissions
3. Look for errors in application logs
4. Test individual API endpoints
```

### Debug Mode

Enable detailed logging:

```bash
# Set environment variables
export RUST_LOG=debug
export TAURI_DEBUG=true

# Run with debug output
npm run tauri:dev
```

### Getting Help

1. **Check Logs**: Application logs are available in the console
2. **Database Status**: Use the ConnectionStatus component to verify database connectivity
3. **Issue Reports**: Submit issues to the project repository
4. **Documentation**: Refer to `CLAUDE.md` for detailed project information

## 🤝 Contributing

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Follow the development workflow outlined above
4. Run tests and ensure all checks pass
5. Submit a pull request

### Code Standards

- **TypeScript**: Strict type checking enabled
- **Rust**: Follow `rustfmt` formatting
- **Comments**: JSDoc for TypeScript, rustdoc for Rust
- **Testing**: Unit tests for business logic
- **Documentation**: Update README and CLAUDE.md for new features

### Commit Guidelines

```bash
git commit -m "feat: add project template copying functionality"
git commit -m "fix: resolve database connection timeout issue"
git commit -m "docs: update API documentation for new endpoints"
```

## 📄 License

This project is proprietary software developed for emittiv. All rights reserved.

## 👥 Support

For technical support or questions:
- **Email**: martin@emittiv.com
- **Repository**: https://git.mms.name/martin/fee-prop.git
- **Documentation**: Refer to `CLAUDE.md` for comprehensive project details

---

**Last Updated**: June 16, 2025  
**Version**: 2.0.0  
**Status**: Production Ready