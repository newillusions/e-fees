# E2E Testing Infrastructure

## Quick Reference

### Running Tests

```bash
# Basic E2E tests
npm run test:e2e

# Interactive test runner
npm run test:e2e:ui

# Development mode (with running dev server)
npm run test:e2e:dev

# Debug mode
npm run test:e2e:debug
```

### Test Categories

| Test File | Coverage | Status |
|-----------|----------|---------|
| `setup.spec.ts` | Environment verification | ✅ Complete |
| `app-startup.spec.ts` | Application launch and initialization | ✅ Complete |
| `project-management.spec.ts` | Project CRUD operations | ✅ Complete |
| `company-contact.spec.ts` | Company/Contact management | ✅ Complete |
| `proposal-workflows.spec.ts` | Proposal creation workflows | 🚧 Planned |
| `navigation.spec.ts` | Keyboard shortcuts and routing | 🚧 Planned |
| `error-scenarios.spec.ts` | Error handling and resilience | 🚧 Planned |

### Key Features

- **Isolated Testing Environment**: Dedicated SurrealDB instance on port 8001
- **Database Fixtures**: Automated test data seeding and cleanup
- **Page Object Model**: Maintainable UI interaction abstractions
- **Tauri Integration**: Native desktop app testing capabilities
- **CI/CD Ready**: GitHub Actions workflow with multi-platform support
- **Comprehensive Reporting**: HTML reports, screenshots, videos, performance metrics

### Quick Start

1. **Prerequisites**: Ensure SurrealDB is installed
2. **Install Dependencies**: `npm ci` (if not already done)
3. **Run Tests**: `npm run test:e2e`

### File Structure

```
e2e/
├── config/          # Setup and teardown
├── fixtures/        # Test data management
├── helpers/         # Page objects and utilities
├── tests/           # Test specifications
├── reports/         # Generated reports
└── test-results/    # Screenshots, videos, traces
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SURREALDB_URL` | `ws://localhost:8001` | Test database URL |
| `SURREALDB_NS` | `test` | Database namespace |
| `SURREALDB_DB` | `e2e` | Database name |
| `SURREALDB_USER` | `root` | Database username |
| `SURREALDB_PASS` | `test` | Database password |
| `PROJECT_FOLDER_PATH` | `/tmp/test-projects` | Test project folders |
| `E2E_DEV_MODE` | `false` | Use running dev server |

### Writing New Tests

1. **Create test file** in `e2e/tests/`
2. **Import required helpers**:
   ```typescript
   import { TauriApp } from '../helpers/tauri-helpers'
   import { NavigationPage } from '../helpers/page-objects'
   import { DatabaseFixtures } from '../fixtures/database-fixtures'
   ```
3. **Follow existing patterns** for setup and teardown
4. **Use page objects** for UI interactions
5. **Manage database state** with fixtures

### Debugging Tests

```bash
# Run with browser visible
npm run test:e2e:headed

# Interactive debugging
npm run test:e2e:debug

# Verbose logging
DEBUG=pw:api npm run test:e2e

# Run specific test
npm run test:e2e -- app-startup.spec.ts
```

### CI/CD Integration

Tests run automatically on:
- Push to `main` or `develop` branches
- Pull requests to `main`
- Manual workflow dispatch

**Platforms tested**: Ubuntu, macOS, Windows

See `.github/workflows/e2e-tests.yml` for complete CI/CD configuration.

### Troubleshooting

**Common Issues:**

1. **SurrealDB not found**: Install SurrealDB and ensure it's in PATH
2. **Port 8001 in use**: Kill conflicting processes or change port
3. **App launch timeout**: Check if app builds successfully
4. **Database connection**: Verify SurrealDB is accessible

**Debug Commands:**
```bash
# Check SurrealDB
curl http://localhost:8001/version

# Check ports
lsof -i :8001

# Manual SurrealDB start
surreal start --user root --pass test --bind 0.0.0.0:8001 memory
```

### Performance

- **Test Execution**: ~2-5 minutes for full suite
- **Database**: In-memory SurrealDB for speed
- **Parallel**: Disabled for desktop app testing
- **Artifacts**: Automatic cleanup after 7 days

### Contributing

1. **Add new tests** following existing patterns
2. **Update page objects** when UI changes
3. **Extend test data** for new scenarios
4. **Document new features** in test comments
5. **Run full test suite** before committing

### Documentation

- **Full Guide**: See `E2E_TESTING_GUIDE.md`
- **API Reference**: Inline code documentation
- **Examples**: Existing test files in `e2e/tests/`

---

For detailed information, see [E2E_TESTING_GUIDE.md](../E2E_TESTING_GUIDE.md)