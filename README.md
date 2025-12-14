# E-Fees - Digital Fee Proposal Management System

<div align="center">
  <img src="src-tauri/icons/icon.png" alt="E-Fees Logo" width="128" height="128">
  
  **Premium Desktop Application for Architecture & Engineering Firms**
  
  Built with Tauri v2 + Svelte 5 + SurrealDB
  
  [![Version](https://img.shields.io/badge/version-0.10.24-blue.svg)]()
  [![License](https://img.shields.io/badge/license-Proprietary-red.svg)]()
  [![Status](https://img.shields.io/badge/status-Production%20Ready-green.svg)]()
</div>

---

## 🎯 Overview

E-Fees is a professional desktop application designed to streamline fee proposal management for architecture and engineering firms. It provides comprehensive tools for managing projects, companies, contacts, and fee proposals with a modern, responsive interface.

## ✨ Key Features

- **📋 Project Management** - Track projects with auto-generated numbering system
- **🏢 Company Database** - Manage client companies and relationships
- **👥 Contact Management** - Maintain contacts linked to companies
- **💼 Fee Proposals** - Create and manage detailed fee proposals
- **🎨 Modern UI** - Dark theme with Emittiv brand design system
- **⚡ Fast & Responsive** - Optimistic updates for instant feedback
- **🔒 Secure** - TLS support and credential management
- **💾 Real-time Database** - SurrealDB with WebSocket connection

## 🚀 Quick Start

### Prerequisites
- Node.js 18+
- Rust 1.70+
- SurrealDB instance

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/e-fees.git
cd e-fees

# Install dependencies
npm install

# Copy environment template
cp .env.template .env
# Edit .env with your database credentials

# Start development server
npm run tauri:dev
```

### Building for Production

```bash
# Create production build
npm run tauri:build

# Output will be in src-tauri/target/release/bundle/
```

## 📚 Documentation

- **[Project Status](PROJECT_STATUS.md)** - Current development status and metrics
- **[Claude Instructions](CLAUDE.md)** - AI assistant integration guide
- **[Development Guide](docs/development/)** - Setup and development documentation
- **[Testing Guide](docs/testing/)** - Testing strategies and E2E with Tauri MCP
- **[Security Guide](docs/security/)** - Security implementation and best practices

## 🛠 Tech Stack

- **Frontend**: Svelte 5 with TypeScript
- **Backend**: Tauri v2 (Rust)
- **Database**: SurrealDB
- **Styling**: TailwindCSS
- **Testing**: Vitest + Tauri MCP
- **Build**: Vite

## ⌨️ Keyboard Shortcuts

- `Cmd/Ctrl + 1` - Dashboard
- `Cmd/Ctrl + 2` - Projects
- `Cmd/Ctrl + 3` - Companies
- `Cmd/Ctrl + 4` - Contacts
- `Cmd/Ctrl + 5` - Proposals
- `Cmd + W` - Position window (4K monitors)

## 🧪 Testing

```bash
# Run unit tests
npm run test

# Run E2E tests with Tauri MCP
npm run test:e2e:mcp

# Clean test data
npm run test:e2e:cleanup

# Type checking
npm run check
```

## 📈 Current Status

- **Version**: 0.10.24 (Production Ready)
- **Auto-Updater**: Working (macOS tested)
- **Database Records**: 48 projects, 37 proposals, 19 companies
- **Performance**: < 2s load time, < 100MB memory

## 🤝 Contributing

This is a proprietary project for Emittiv. For internal contributors:

1. Create a feature branch from `main`
2. Make your changes with appropriate tests
3. Ensure all tests pass
4. Submit a pull request for review

## 📄 License

Proprietary - © 2025 Emittiv. All rights reserved.

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app)
- UI framework by [Svelte](https://svelte.dev)
- Database by [SurrealDB](https://surrealdb.com)
- Design system by Emittiv

---

<div align="center">
  <strong>Built with ❤️ by Emittiv</strong>
</div>