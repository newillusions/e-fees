# Commands Quick Reference

> **Purpose**: Fast lookup for common commands
> **Use**: Keep this open during development sessions

---

## 🚀 Development

```bash
# Start development server
npm run tauri:dev

# Start frontend only (no Tauri)
npm run dev

# Build for production
npm run tauri:build

# Quick Rust check (no build)
cargo check

# Type check TypeScript
npm run check
```

---

## 🧪 Testing

```bash
# Run all tests
npm test

# Run E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e:ui

# Run specific test
npm run test:e2e -- --grep "test-name"

# Run tests in watch mode
npm run test:watch

# Coverage report
npm run test:coverage
```

### E2E Test Data Management

```bash
# List test data
npm run test:e2e:list-test-data

# Clean up test data
npm run test:e2e:cleanup

# Verify cleanup
npm run test:e2e:verify-clean

# Emergency cleanup
npm run test:e2e:emergency-cleanup

# Safe E2E run (test + cleanup + verify)
npm run test:e2e:safe
```

---

## 💾 Database

```bash
# Open Surrealist (DB GUI)
npm run db:studio

# Initialize database
npm run db:init

# Check database connection
surreal isready --conn file:./data/efees.db

# Backup database
cp -r ./data/efees.db ./data/efees.db.backup.$(date +%s)
```

---

## 🔍 MCP & Debugging

```bash
# Check MCP socket exists
ls -la /tmp/tauri-mcp-e2e.sock

# Kill MCP processes
pkill -f tauri-mcp

# Remove stale socket
rm -f /tmp/tauri-mcp-e2e.sock

# Debug mode (with logs)
RUST_LOG=debug npm run tauri:dev

# Full debug (backtrace)
RUST_LOG=debug RUST_BACKTRACE=1 npm run tauri:dev
```

---

## 🔧 Troubleshooting

```bash
# === Socket Issues ===
pkill -f tauri-mcp && rm /tmp/tauri-mcp-e2e.sock && npm run tauri:dev

# === Build Issues ===
cargo clean && rm -rf target/ && cargo build

# === Frontend Issues ===
rm -rf node_modules/.vite && npm run dev

# === Nuclear Option (resets everything) ===
pkill -f tauri-mcp && \
rm -rf ./data/efees.db /tmp/tauri-mcp-e2e.sock target/ node_modules/.vite && \
cargo clean && npm run db:init
```

---

## 📦 Dependencies

```bash
# Install dependencies
npm install

# Install Rust dependencies
cargo fetch

# Update dependencies
npm update
cargo update

# Clean install
rm -rf node_modules/ package-lock.json && npm install
```

---

## 🔎 Search & Find

```bash
# Find files by name
find . -name "*.ts" -not -path "*/node_modules/*"

# Search in code
grep -r "searchTerm" src/

# Search with ripgrep (faster)
rg "searchTerm" src/

# Find in Rust code
grep -r "searchTerm" src-tauri/src/

# Find large files
find . -type f -size +1M -not -path "*/node_modules/*"
```

---

## 📊 Diagnostics

```bash
# Check running processes
ps aux | grep -i "e-fees"

# Check MCP status
lsof /tmp/tauri-mcp-e2e.sock

# View recent logs
tail -50 src-tauri/target/debug/app.log

# Monitor logs in real-time
tail -f src-tauri/target/debug/app.log

# Check port usage
lsof -i :1421
```

---

## 🌿 Git

```bash
# Status
git status

# Recent commits
git log --oneline -10

# Staged changes
git diff --staged

# Unstaged changes
git diff

# Create branch
git checkout -b feat/feature-name

# Commit
git add .
git commit -m "type(scope): message"

# Push
git push origin branch-name
```

---

## 🏗️ Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build frontend only
npm run build

# Full production build
npm run tauri:build

# Build with timing
time npm run tauri:build
```

---

## 🔐 Environment

```bash
# Load environment variables
source .env.local

# Verify loaded
env | grep SURREAL

# Check specific variable
echo $SURREALDB_URL
```

---

## 📏 Code Quality

```bash
# Format Rust code
cargo fmt

# Rust linter
cargo clippy

# Fix clippy warnings
cargo clippy --fix

# Format TypeScript
npm run format

# Lint TypeScript
npm run lint

# Fix lint issues
npm run lint:fix
```

---

## 💻 System

```bash
# Check versions
node --version
npm --version
cargo --version
rustc --version

# Disk usage
du -sh ./data/efees.db
du -sh ./target
du -sh ./node_modules

# Free up space
cargo clean
rm -rf node_modules/.vite
rm -rf target/
```

---

## 🎯 Quick Fixes

```bash
# App won't start
cargo clean && cargo build

# Socket error
pkill -f tauri-mcp && rm /tmp/tauri-mcp-e2e.sock

# Database corruption
rm -rf ./data/efees.db && npm run db:init

# Frontend blank
rm -rf node_modules/.vite && npm run dev

# Build error
cargo clean && rm -rf target/ node_modules/ && npm install
```

---

## 📚 Help

```bash
# npm scripts available
npm run

# Cargo commands
cargo --help

# Tauri CLI help
npm run tauri -- --help

# Specific command help
npm run tauri build -- --help
```

---

## 🔗 Quick Links

- **Full documentation**: [claude.md](../claude.md)
- **Navigation**: [NAVIGATOR.md](../NAVIGATOR.md)
- **Troubleshooting**: [claude.md#troubleshooting](../claude.md#quick-troubleshooting)
- **MCP issues**: [MCP-SERVERS.md](../MCP-SERVERS.md)

---

**Pro Tip**: Add frequently used commands to shell aliases:

```bash
# Add to ~/.zshrc or ~/.bashrc
alias efees-dev="cd /Volumes/base/dev/e-fees && npm run tauri:dev"
alias efees-test="cd /Volumes/base/dev/e-fees && npm run test:e2e"
alias efees-clean="cd /Volumes/base/dev/e-fees && npm run test:e2e:cleanup"
```
