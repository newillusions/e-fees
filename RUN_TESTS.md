# Running Unit Tests - Quick Reference Guide

## Quick Start

```bash
# Run ALL tests (Rust + TypeScript)
npm run test:all

# Or run separately:
cd src-tauri && cargo test              # Rust tests
cd .. && npm run test:run               # TypeScript tests
```

---

## Rust Backend Tests

### Location
`/Volumes/base/dev/e-fees/src-tauri/src/db/tests.rs`

### Commands

```bash
# Navigate to Rust project
cd /Volumes/base/dev/e-fees/src-tauri

# Run all Rust tests
cargo test

# Run with output (see println! statements)
cargo test -- --nocapture

# Run specific test module
cargo test db::tests

# Run specific test
cargo test test_sql_escape_single_quotes

# Run tests matching a pattern
cargo test project_number

# Run tests in release mode (faster)
cargo test --release

# Show test execution time
cargo test -- --show-output
```

### Expected Output

```
running 23 tests
test db::tests::test_project_number_validation_valid_formats ... ok
test db::tests::test_project_number_validation_invalid_formats ... ok
test db::tests::test_project_number_parsing_success ... ok
test db::tests::test_project_number_parsing_edge_cases ... ok
test db::tests::test_project_number_parsing_errors ... ok
test db::tests::test_project_number_range_validation ... ok
test db::tests::test_sql_escape_single_quotes ... ok
test db::tests::test_sql_escape_special_characters ... ok
test db::tests::test_sql_injection_attack_patterns ... ok
test db::tests::test_database_config_from_env_success ... ok
test db::tests::test_database_config_missing_required_vars ... ok
test db::tests::test_database_config_tls_settings ... ok
test db::tests::test_format_project_number ... ok
test db::tests::test_increment_sequence ... ok
test db::tests::test_extract_thing_id ... ok
test db::tests::test_extract_thing_id_from_string ... ok
test db::tests::test_validate_email_format ... ok
test db::tests::test_validate_phone_format ... ok
test db::tests::test_validate_country_code ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## TypeScript Frontend Tests

### Location
`/Volumes/base/dev/e-fees/src/lib/api.test.ts`

### Commands

```bash
# Navigate to project root
cd /Volumes/base/dev/e-fees

# Run all TypeScript tests once
npm run test:run

# Run with coverage report
npm run test:coverage

# Run in watch mode (re-run on file changes)
npm run test:watch

# Run with UI (browser-based interface)
npm run test:ui

# Run specific test file
npm run test:run src/lib/api.test.ts

# Run tests matching a pattern
npm run test:run -- --grep "connection"
```

### Expected Output

```
✓ src/lib/api.test.ts (60 tests) 2145ms
  ✓ Database Connection (4)
    ✓ checkDbConnection › should return true when connection is successful
    ✓ checkDbConnection › should return false when connection fails
    ✓ getConnectionStatus › should return connection status when successful
    ✓ getConnectionStatus › should return disconnected status on error
  ✓ Project CRUD Operations (6)
    ✓ getProjects › should return projects array when successful
    ✓ searchProjects › should return filtered projects for valid query
    ✓ createProject › should create project successfully
    ... (57 more tests)

Test Files  1 passed (1)
     Tests  60 passed (60)
  Start at  12:34:56
  Duration  3.21s
```

### Coverage Report

After running `npm run test:coverage`:

```
File                | % Stmts | % Branch | % Funcs | % Lines
--------------------|---------|----------|---------|--------
src/lib/api.ts      |   88.2  |   85.3   |  90.1   |  88.4

Coverage report available at: coverage/index.html
```

Open the coverage report in your browser:
```bash
open coverage/index.html
```

---

## Continuous Integration

### GitHub Actions Workflow

Create `.github/workflows/tests.yml`:

```yaml
name: Unit Tests

on:
  push:
    branches: [ main, dev ]
  pull_request:
    branches: [ main, dev ]

jobs:
  rust-tests:
    name: Rust Unit Tests
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Run tests
        run: cd src-tauri && cargo test --verbose

  typescript-tests:
    name: TypeScript Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Run tests with coverage
        run: npm run test:coverage

      - name: Upload coverage reports
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
          flags: unittests
          name: codecov-umbrella
```

---

## Troubleshooting

### Rust Tests Failing

**Issue**: Tests compile but fail at runtime

**Common Causes**:
1. Missing environment variables
2. Database connection issues
3. File system permissions

**Solutions**:

```bash
# Check compilation
cargo check

# Run with backtrace for detailed errors
RUST_BACKTRACE=1 cargo test

# Run single test for debugging
cargo test test_name -- --exact --nocapture
```

### TypeScript Tests Failing

**Issue**: Tests fail with "Module not found" or similar

**Solutions**:

```bash
# Reinstall dependencies
rm -rf node_modules package-lock.json
npm install

# Clear Vite cache
rm -rf node_modules/.vite

# Verify TypeScript configuration
npx tsc --noEmit
```

### Coverage Not Generating

**Issue**: Coverage report is empty or missing

**Solutions**:

```bash
# Install coverage dependencies
npm install --save-dev @vitest/coverage-v8

# Run with explicit coverage provider
npm run test:coverage -- --coverage.provider=v8

# Check coverage directory
ls -la coverage/
```

---

## Test Development Workflow

### 1. Write Test First (TDD)

```rust
// Rust example
#[test]
fn test_new_feature() {
    let result = new_feature();
    assert_eq!(result, expected_value);
}
```

```typescript
// TypeScript example
it('should do something new', async () => {
  mockInvoke.mockResolvedValueOnce(expectedData)
  const result = await ApiClient.newMethod()
  expect(result).toEqual(expectedData)
})
```

### 2. Run Test (Should Fail)

```bash
cargo test test_new_feature
# or
npm run test:watch
```

### 3. Implement Feature

```rust
// Implement the function
pub fn new_feature() -> Type {
    // implementation
}
```

### 4. Run Test (Should Pass)

```bash
cargo test test_new_feature
# or
# Test should auto-run in watch mode
```

### 5. Refactor & Verify

```bash
# Run all tests to ensure nothing broke
cargo test
npm run test:run
```

---

## Performance Monitoring

### Test Execution Time

```bash
# Rust - Show timing
cargo test -- --show-output

# TypeScript - Show timing
npm run test:run -- --reporter=verbose
```

### Benchmark Tests (Future)

```bash
# Rust benchmarks (requires nightly)
cargo +nightly bench

# TypeScript benchmarks
npm run test:run -- --benchmark
```

---

## Best Practices

### 1. Run Tests Before Commit

```bash
# Pre-commit hook (.git/hooks/pre-commit)
#!/bin/bash
cd src-tauri && cargo test
cd .. && npm run test:run

if [ $? -ne 0 ]; then
    echo "Tests failed. Commit aborted."
    exit 1
fi
```

### 2. Watch Mode During Development

```bash
# Terminal 1: TypeScript watch
npm run test:watch

# Terminal 2: Rust watch (requires cargo-watch)
cargo install cargo-watch
cd src-tauri && cargo watch -x test
```

### 3. Coverage Thresholds

Maintain minimum coverage:
- **Critical paths**: 100% (SQL injection prevention, validation)
- **Business logic**: 90% (project numbering, calculations)
- **Utilities**: 85% (helpers, formatters)
- **Overall**: 80%

Check coverage:
```bash
npm run test:coverage
# Review coverage/index.html
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Run all tests | `npm run test:all` |
| Rust tests only | `cd src-tauri && cargo test` |
| TypeScript tests only | `npm run test:run` |
| Coverage report | `npm run test:coverage` |
| Watch mode | `npm run test:watch` |
| Specific test | `cargo test test_name` |
| Debug mode | `RUST_BACKTRACE=1 cargo test` |
| UI mode | `npm run test:ui` |

---

## Getting Help

### Documentation
- Rust Testing Book: https://doc.rust-lang.org/book/ch11-00-testing.html
- Vitest Docs: https://vitest.dev/
- Project Testing Strategy: `.claude/context/testing-strategy.md`

### Common Questions

**Q: How do I run just one test?**
```bash
# Rust
cargo test test_name -- --exact

# TypeScript
npm run test:run -- --grep "test description"
```

**Q: How do I see console output in tests?**
```bash
# Rust
cargo test -- --nocapture

# TypeScript
# Output is shown by default in Vitest
```

**Q: How do I debug a failing test?**
```bash
# Rust
RUST_BACKTRACE=full cargo test test_name -- --exact --nocapture

# TypeScript
npm run test:ui
# Then click the failing test
```

---

**Last Updated**: November 16, 2025
**Next Update**: After first test run verification
