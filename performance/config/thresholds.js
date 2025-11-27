/**
 * Performance Thresholds Configuration for E-Fees Application
 * 
 * Defines performance budgets, regression thresholds, and alerting criteria
 * for different aspects of the application performance.
 */

// Base performance thresholds for different environments
export const environmentThresholds = {
  development: {
    // Relaxed thresholds for development environment
    responseTime: {
      p50: 200,   // 50% under 200ms
      p90: 500,   // 90% under 500ms  
      p95: 1000,  // 95% under 1000ms
      p99: 2000,  // 99% under 2000ms
    },
    errorRate: 0.05, // 5% error rate allowed
    throughput: 10,  // 10 requests/second minimum
    memory: {
      baseline: 150,    // 150MB baseline
      working: 400,     // 400MB working set
      growth: 5,        // 5MB/minute growth
    }
  },

  testing: {
    // Moderate thresholds for testing environment
    responseTime: {
      p50: 100,
      p90: 300,
      p95: 500,
      p99: 1000,
    },
    errorRate: 0.02, // 2% error rate
    throughput: 50,  // 50 requests/second
    memory: {
      baseline: 120,
      working: 300,
      growth: 3,
    }
  },

  production: {
    // Strict thresholds for production-like performance
    responseTime: {
      p50: 50,
      p90: 150,
      p95: 300,
      p99: 500,
    },
    errorRate: 0.01, // 1% error rate
    throughput: 100, // 100 requests/second
    memory: {
      baseline: 100,
      working: 250,
      growth: 1,
    }
  }
};

// Component-specific performance thresholds
export const componentThresholds = {
  // Database operation thresholds
  database: {
    singleQuery: {
      select: 50,     // Single SELECT under 50ms
      insert: 100,    // INSERT under 100ms
      update: 100,    // UPDATE under 100ms
      delete: 75,     // DELETE under 75ms
    },
    bulkOperations: {
      insert: 1000,   // 1000 inserts/second
      update: 500,    // 500 updates/second
      delete: 800,    // 800 deletes/second
      select: 2000,   // 2000 selects/second
    },
    complexQueries: {
      simpleJoin: 100,      // 2-table join under 100ms
      multipleJoin: 300,    // 3+ table join under 300ms
      aggregation: 200,     // Aggregations under 200ms
      fullTextSearch: 500,  // Search under 500ms
    },
    connectionPool: {
      maxConnections: 50,
      connectionTime: 1000, // Connect under 1 second
      idleTimeout: 30000,   // 30 second idle timeout
    }
  },

  // UI/Frontend performance thresholds
  frontend: {
    rendering: {
      firstPaint: 800,        // First paint under 800ms
      meaningfulPaint: 1500,  // Meaningful paint under 1.5s
      interactive: 3000,      // Interactive under 3s
      frameRate: 60,          // 60fps target
      frameBudget: 16.67,     // 16.67ms per frame (60fps)
    },
    components: {
      modalOpen: 100,         // Modal open under 100ms
      modalClose: 50,         // Modal close under 50ms  
      formSubmit: 200,        // Form submit under 200ms
      searchFilter: 100,      // Search filter under 100ms
      listScroll: 16,         // Smooth scrolling (60fps)
    },
    dataLoading: {
      smallDataset: 200,      // <100 items under 200ms
      mediumDataset: 500,     // 100-1000 items under 500ms
      largeDataset: 1500,     // 1000+ items under 1.5s
    },
    memory: {
      componentMemory: 5,     // 5MB per major component
      memoryLeakRate: 0.1,    // <0.1MB/minute leak rate
      garbageCollection: 50,  // GC under 50ms
    }
  },

  // Workflow-specific thresholds
  workflows: {
    // Project management workflows
    projectWorkflows: {
      createProject: 1000,    // Create project under 1s
      updateProject: 500,     // Update project under 500ms
      deleteProject: 300,     // Delete project under 300ms
      viewProject: 200,       // View project under 200ms
      searchProjects: 300,    // Search projects under 300ms
    },
    
    // Proposal management workflows  
    proposalWorkflows: {
      createProposal: 1500,   // Create proposal under 1.5s
      updateProposal: 800,    // Update proposal under 800ms
      deleteProposal: 400,    // Delete proposal under 400ms
      viewProposal: 300,      // View proposal under 300ms
      searchProposals: 400,   // Search proposals under 400ms
    },

    // Company/Contact workflows
    crmWorkflows: {
      createCompany: 800,     // Create company under 800ms
      createContact: 600,     // Create contact under 600ms
      updateCompany: 400,     // Update company under 400ms
      updateContact: 300,     // Update contact under 300ms
      searchCRM: 200,         // CRM search under 200ms
    }
  },

  // Application startup and navigation
  application: {
    startup: {
      cold: 3000,             // Cold start under 3s
      warm: 1000,             // Warm start under 1s  
      dataLoad: 2000,         // Initial data load under 2s
      firstInteraction: 4000, // First interaction under 4s
    },
    navigation: {
      routeChange: 200,       // Route change under 200ms
      pageLoad: 500,          // Page load under 500ms
      backButton: 100,        // Back navigation under 100ms
    }
  }
};

// Performance regression thresholds
export const regressionThresholds = {
  // Percentage increase that triggers alerts
  warning: {
    responseTime: 15,       // 15% increase warns
    errorRate: 50,          // 50% increase in errors warns
    memory: 20,             // 20% memory increase warns
    throughput: -10,        // 10% throughput decrease warns
  },
  
  critical: {
    responseTime: 30,       // 30% increase fails build
    errorRate: 100,         // 100% increase in errors fails build
    memory: 40,             // 40% memory increase fails build  
    throughput: -25,        // 25% throughput decrease fails build
  },

  // Absolute thresholds that always fail regardless of baseline
  absolute: {
    responseTime: 5000,     // Never accept >5s response
    errorRate: 0.1,         // Never accept >10% error rate
    memory: 1000,           // Never accept >1GB memory
    downtime: 0.001,        // Never accept >0.1% downtime
  }
};

// Performance budgets for different data scales
export const performanceBudgets = {
  // Small dataset (500 projects, 1000 proposals)
  small: {
    database: {
      queryTime: 50,
      resultSize: 100,        // 100 items per query
      memoryUsage: 50,        // 50MB database memory
    },
    frontend: {
      renderTime: 100,
      listSize: 500,          // 500 items in lists
      memoryUsage: 100,       // 100MB frontend memory
    }
  },

  // Medium dataset (2500 projects, 5000 proposals) 
  medium: {
    database: {
      queryTime: 150,
      resultSize: 500,        // 500 items per query
      memoryUsage: 150,       // 150MB database memory
    },
    frontend: {
      renderTime: 300,
      listSize: 1000,         // 1000 items in lists
      memoryUsage: 200,       // 200MB frontend memory
    }
  },

  // Large dataset (10000 projects, 25000 proposals)
  large: {
    database: {
      queryTime: 500,
      resultSize: 1000,       // 1000 items per query
      memoryUsage: 300,       // 300MB database memory
    },
    frontend: {
      renderTime: 1000,
      listSize: 2000,         // 2000 items in lists (virtualized)
      memoryUsage: 400,       // 400MB frontend memory
    }
  }
};

// k6 threshold configurations for different test types
export const k6Thresholds = {
  smoke: {
    // Basic functionality validation
    'http_req_duration': ['p(95)<1000'],
    'http_req_failed': ['rate<0.05'],
    'checks': ['rate>0.95'],
  },
  
  load: {
    // Normal load conditions
    'http_req_duration': ['p(50)<200', 'p(95)<500'],
    'http_req_duration{operation:database}': ['p(90)<100'],
    'http_req_duration{operation:search}': ['p(95)<300'],
    'http_req_failed': ['rate<0.02'],
    'http_reqs': ['rate>50'],
    'checks': ['rate>0.98'],
  },

  stress: {
    // High load conditions
    'http_req_duration': ['p(50)<500', 'p(95)<2000'],
    'http_req_duration{operation:database}': ['p(90)<300'],
    'http_req_failed': ['rate<0.1'],
    'http_reqs': ['rate>20'],
    'checks': ['rate>0.90'],
  },

  spike: {
    // Sudden load spikes
    'http_req_duration': ['p(50)<1000', 'p(95)<5000'],
    'http_req_failed': ['rate<0.2'],
    'checks': ['rate>0.85'],
  },

  endurance: {
    // Long-running stability
    'http_req_duration': ['p(95)<500'],
    'http_req_failed': ['rate<0.01'],
    'memory_growth_mb_per_min': ['avg<2'],
    'checks': ['rate>0.99'],
  }
};

// Custom metric thresholds for E-Fees specific operations
export const customThresholds = {
  // SurrealDB specific metrics
  surrealdb: {
    'surrealdb_query_duration': ['p(95)<200'],
    'surrealdb_connection_time': ['p(90)<1000'],
    'surrealdb_websocket_reconnects': ['rate<0.001'],
    'surrealdb_result_size_kb': ['avg<500'],
  },

  // Tauri specific metrics
  tauri: {
    'tauri_command_duration': ['p(95)<100'],
    'tauri_ipc_message_size': ['avg<100'],
    'tauri_startup_time': ['p(90)<3000'],
    'tauri_memory_usage_mb': ['avg<200'],
  },

  // Svelte specific metrics
  svelte: {
    'svelte_component_mount_time': ['p(95)<50'],
    'svelte_state_update_time': ['p(95)<10'],
    'svelte_reactive_statement_time': ['p(99)<5'],
    'svelte_component_memory_mb': ['avg<10'],
  },

  // Business workflow metrics
  business: {
    'project_creation_time': ['p(95)<1000'],
    'proposal_search_time': ['p(95)<200'],
    'company_lookup_time': ['p(90)<100'],
    'export_generation_time': ['p(95)<5000'],
  }
};

// Helper functions for threshold management
export const thresholdUtils = {
  // Get thresholds for specific environment
  getEnvironmentThresholds(environment = 'testing') {
    return environmentThresholds[environment] || environmentThresholds.testing;
  },

  // Get performance budget for dataset size
  getPerformanceBudget(datasetSize = 'medium') {
    return performanceBudgets[datasetSize] || performanceBudgets.medium;
  },

  // Check if metric passes threshold
  checkThreshold(metric, value, thresholdConfig) {
    const threshold = thresholdConfig[metric];
    if (!threshold) return true;
    
    if (typeof threshold === 'number') {
      return value <= threshold;
    }
    
    if (threshold.max !== undefined) {
      return value <= threshold.max;
    }
    
    return true;
  },

  // Calculate regression percentage
  calculateRegression(baseline, current) {
    if (!baseline || baseline === 0) return 0;
    return ((current - baseline) / baseline) * 100;
  },

  // Check if regression exceeds threshold
  isRegressionCritical(baseline, current, metric) {
    const regression = this.calculateRegression(baseline, current);
    const warning = regressionThresholds.warning[metric];
    const critical = regressionThresholds.critical[metric];
    
    return {
      regression,
      isWarning: Math.abs(regression) > warning,
      isCritical: Math.abs(regression) > critical,
      isAbsoluteFail: current > regressionThresholds.absolute[metric]
    };
  }
};

export default {
  environmentThresholds,
  componentThresholds, 
  regressionThresholds,
  performanceBudgets,
  k6Thresholds,
  customThresholds,
  thresholdUtils
};