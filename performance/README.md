# E-Fees Performance Testing Infrastructure

## Overview

Comprehensive performance testing framework for the E-Fees desktop application, designed to validate performance with large datasets and identify bottlenecks across database operations, UI rendering, and user interactions.

## Architecture

### 1. **Testing Layers**
- **Database Performance**: SurrealDB operations, query optimization, connection handling
- **API Performance**: Tauri command execution, data serialization/deserialization
- **UI Performance**: Svelte component rendering, large list virtualization, search/filtering
- **Memory Performance**: Memory leak detection, garbage collection analysis
- **Integration Performance**: End-to-end user workflows under load

### 2. **Technology Stack**
- **Primary Framework**: k6 (JavaScript-based, excellent for desktop apps)
- **Database Testing**: Custom SurrealDB performance scripts
- **Browser Testing**: Playwright with performance APIs
- **Memory Monitoring**: Node.js built-in profilers + custom metrics
- **Reporting**: HTML dashboard with regression tracking

### 3. **Test Data Scaling**
- **Current Production**: 48 projects, 37 proposals, 19 companies, ~50 contacts
- **Performance Targets**:
  - Small Scale: 500 projects, 1,000 proposals, 100 companies, 500 contacts
  - Medium Scale: 2,000 projects, 5,000 proposals, 500 companies, 2,000 contacts
  - Large Scale: 10,000 projects, 25,000 proposals, 2,000 companies, 10,000 contacts

## Performance Testing Scenarios

### Database Performance Tests
- **Bulk CRUD Operations**: Insert/update/delete 1000+ records
- **Complex Queries**: Multi-table joins with large result sets
- **Search Performance**: Full-text search across all tables
- **Connection Pooling**: High-concurrency database access
- **Index Optimization**: Query performance with/without proper indexes

### UI Performance Tests
- **Large List Rendering**: 5000+ items in project/proposal lists
- **Search Responsiveness**: Real-time filtering with 10,000+ records
- **Modal Performance**: Form handling with complex validation
- **Navigation Speed**: Route transitions under memory pressure
- **Memory Leaks**: Long-running sessions with repeated actions

### User Workflow Tests
- **Startup Performance**: App launch time with different dataset sizes
- **Data Loading**: Time to first meaningful paint
- **CRUD Workflows**: Complete create-read-update-delete cycles
- **Search and Filter**: Complex filtering across multiple criteria
- **Export Operations**: Large dataset export functionality

## Quick Start

```bash
# Install performance testing dependencies
npm run perf:install

# Generate large test dataset
npm run perf:generate-data

# Run all performance tests
npm run perf:test

# Run specific test suites
npm run perf:test:database
npm run perf:test:ui
npm run perf:test:workflows

# Generate performance report
npm run perf:report
```

## Performance Targets

### Response Time Targets
- **Database Queries**: < 100ms for single record, < 500ms for list views
- **UI Updates**: < 16ms frame time (60fps), < 100ms search filtering
- **Modal Operations**: < 200ms open/close, < 50ms form validation
- **Navigation**: < 300ms route transitions
- **Startup Time**: < 3 seconds on HDD, < 1.5 seconds on SSD

### Memory Targets
- **Base Memory**: < 100MB on startup
- **Working Set**: < 300MB with 5000+ records loaded
- **Memory Growth**: < 10MB per hour in idle state
- **Peak Usage**: < 500MB during bulk operations

### Throughput Targets
- **Database Operations**: 100+ queries/second
- **UI Updates**: Handle 1000+ items without blocking
- **Search**: Filter 10,000+ records in < 200ms
- **File Operations**: Export 5000+ records in < 5 seconds

## Test Environment

### Development Environment
- **Hardware**: Various configurations (M1 Mac, Intel Windows, Linux)
- **Database**: Local SurrealDB instance with performance data
- **Network**: Local WebSocket connections (minimal latency)

### CI/CD Environment  
- **Hardware**: Standardized GitHub Actions runners
- **Database**: Docker SurrealDB with consistent performance
- **Reporting**: Automated regression detection and alerting

## Directory Structure

```
performance/
├── README.md                 # This file
├── config/                   # Test configurations
│   ├── k6.config.js         # k6 performance test config
│   ├── database.config.js   # SurrealDB test config
│   └── thresholds.js        # Performance thresholds
├── data/                     # Test data generation
│   ├── generators/          # Data generation scripts
│   ├── fixtures/           # Realistic test data templates
│   └── scaling/            # Large dataset creation
├── tests/                   # Performance test suites
│   ├── database/           # Database performance tests
│   ├── ui/                 # UI rendering performance
│   ├── workflows/          # End-to-end workflows
│   ├── memory/             # Memory leak detection
│   └── regression/         # Performance regression tests
├── utils/                   # Testing utilities
│   ├── reporters/          # Custom result reporting
│   ├── monitoring/         # Performance monitoring
│   └── helpers/            # Test helper functions
├── reports/                 # Generated performance reports
│   ├── html/               # Interactive HTML dashboards
│   ├── json/               # Raw performance data
│   └── trends/             # Historical performance trends
└── scripts/                 # Automation scripts
    ├── setup.sh            # Environment setup
    ├── run-tests.sh        # Test execution
    └── generate-report.sh  # Report generation
```

## Continuous Performance Testing

### Regression Detection
- Automated performance baseline comparison
- Alert on >10% performance degradation
- Track performance trends over time
- Compare branch performance vs main

### Performance Budgets
- Enforce maximum response times
- Monitor memory usage growth
- Track bundle size increases  
- Alert on threshold violations

### Integration with CI/CD
- Run performance tests on every PR
- Generate performance comparison reports
- Block deployments on critical regressions
- Store historical performance data

---

**Next Steps**: Run `npm run perf:install` to set up the performance testing environment.