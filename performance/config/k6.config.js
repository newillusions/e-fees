/**
 * k6 Performance Testing Configuration for E-Fees Application
 * 
 * This configuration defines performance testing scenarios, thresholds,
 * and environments for comprehensive performance validation.
 */

export const options = {
  scenarios: {
    // Database performance testing - high load, focused on DB operations
    database_stress: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '30s', target: 10 },   // Warm up
        { duration: '2m', target: 50 },    // Ramp to moderate load
        { duration: '3m', target: 100 },   // High load
        { duration: '1m', target: 150 },   // Peak load
        { duration: '2m', target: 0 },     // Cool down
      ],
      gracefulRampDown: '30s',
      tags: { test_type: 'database' },
    },

    // UI performance testing - fewer VUs, focus on rendering performance
    ui_performance: {
      executor: 'constant-vus',
      vus: 5,
      duration: '5m',
      tags: { test_type: 'ui' },
    },

    // Workflow testing - realistic user scenarios
    user_workflows: {
      executor: 'per-vu-iterations',
      vus: 20,
      iterations: 10,
      maxDuration: '10m',
      tags: { test_type: 'workflows' },
    },

    // Memory leak testing - long-running scenario
    memory_stability: {
      executor: 'constant-vus',
      vus: 2,
      duration: '30m',
      tags: { test_type: 'memory' },
    },

    // Spike testing - sudden load increases
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '1m', target: 10 },    // Normal load
        { duration: '10s', target: 100 },  // Sudden spike
        { duration: '2m', target: 100 },   // Maintain spike
        { duration: '10s', target: 10 },   // Quick recovery
        { duration: '1m', target: 10 },    // Settle
      ],
      tags: { test_type: 'spike' },
    },
  },

  // Performance thresholds - fail tests if not met
  thresholds: {
    // Overall response times
    'http_req_duration': ['p(95)<500'], // 95% of requests under 500ms
    'http_req_duration{test_type:database}': ['p(90)<100'], // DB queries under 100ms
    'http_req_duration{test_type:ui}': ['p(95)<200'], // UI operations under 200ms
    
    // Error rates
    'http_req_failed': ['rate<0.01'], // Less than 1% error rate
    'http_req_failed{test_type:database}': ['rate<0.005'], // DB errors < 0.5%
    
    // Custom metrics (defined in test scripts)
    'search_response_time': ['p(95)<200'], // Search under 200ms
    'modal_open_time': ['p(90)<100'], // Modal open under 100ms
    'list_render_time': ['p(95)<300'], // List rendering under 300ms
    'memory_usage_mb': ['avg<300'], // Average memory under 300MB
    'memory_growth_mb_per_min': ['avg<2'], // Memory growth under 2MB/min
    
    // Throughput requirements
    'http_reqs': ['rate>100'], // At least 100 requests/second overall
    'iterations': ['rate>50'], // At least 50 iterations/second for workflows
    
    // Availability
    'checks': ['rate>0.99'], // 99% of checks should pass
  },

  // Test execution options
  userAgent: 'E-Fees-Performance-Test/1.0',
  
  // Disable default metrics we don't need
  noConnectionReuse: false,
  noVUConnectionReuse: false,
  
  // Summary configuration
  summaryTrendStats: ['avg', 'min', 'med', 'max', 'p(90)', 'p(95)', 'p(99)'],
  
  // External metrics and monitoring
  ext: {
    loadimpact: {
      // Cloud monitoring configuration (if using k6 Cloud)
      projectID: parseInt(__ENV.K6_PROJECT_ID || '0'),
      name: 'E-Fees Performance Test',
    },
  },
};

// Environment-specific configurations
export const environments = {
  development: {
    baseUrl: 'http://localhost:1420', // Tauri dev server
    databaseUrl: 'ws://10.0.1.17:8000',
    namespace: 'emittiv',
    database: 'projects',
    credentials: {
      username: 'martin',
      password: process.env.SURREALDB_PASSWORD || 'test_password'
    },
    iterations: {
      small: 10,
      medium: 50,
      large: 100,
    },
  },

  testing: {
    baseUrl: 'http://localhost:3000', // Test environment
    databaseUrl: 'ws://localhost:8000',
    namespace: 'emittiv_test',
    database: 'projects_test',
    credentials: {
      username: 'test_user',
      password: process.env.SURREALDB_TEST_PASSWORD || 'test_password'
    },
    iterations: {
      small: 50,
      medium: 200,
      large: 500,
    },
  },

  ci: {
    baseUrl: 'http://localhost:1420',
    databaseUrl: 'ws://localhost:8000',
    namespace: 'emittiv_ci',
    database: 'projects_ci',
    credentials: {
      username: 'ci_user',
      password: process.env.SURREALDB_CI_PASSWORD || 'ci_password'
    },
    iterations: {
      small: 20,
      medium: 100,
      large: 200,
    },
  },
};

// Get current environment configuration
export function getEnvironment() {
  const env = __ENV.TEST_ENV || 'development';
  return environments[env] || environments.development;
}

// Performance test categories and their specific options
export const testCategories = {
  smoke: {
    // Quick smoke tests - basic functionality
    vus: 1,
    duration: '30s',
    thresholds: {
      'http_req_duration': ['p(95)<1000'], // Relaxed thresholds
      'http_req_failed': ['rate<0.05'],
    },
  },
  
  load: {
    // Normal load testing - expected production traffic
    vus: 10,
    duration: '5m',
    thresholds: {
      'http_req_duration': ['p(95)<500'],
      'http_req_failed': ['rate<0.02'],
    },
  },
  
  stress: {
    // Stress testing - beyond normal capacity
    vus: 50,
    duration: '10m',
    thresholds: {
      'http_req_duration': ['p(95)<1000'], // Allow degradation
      'http_req_failed': ['rate<0.1'],
    },
  },
  
  spike: {
    // Spike testing - sudden traffic increases
    stages: [
      { duration: '1m', target: 5 },
      { duration: '10s', target: 50 },
      { duration: '2m', target: 50 },
      { duration: '10s', target: 5 },
    ],
    thresholds: {
      'http_req_duration': ['p(95)<2000'], // Higher tolerance
      'http_req_failed': ['rate<0.05'],
    },
  },
  
  endurance: {
    // Long-running stability testing
    vus: 5,
    duration: '30m',
    thresholds: {
      'http_req_duration': ['p(95)<500'],
      'http_req_failed': ['rate<0.01'],
      'memory_growth_mb_per_min': ['avg<1'], // Strict memory growth
    },
  },
};

// Helper function to get test category configuration
export function getTestCategory(category = 'load') {
  return testCategories[category] || testCategories.load;
}

// Custom check functions for E-Fees specific validations
export const customChecks = {
  validateSurrealDBResponse: (response) => {
    const body = JSON.parse(response.body);
    return {
      'response is array': Array.isArray(body),
      'no SQL errors': !body.error,
      'has result': body.result !== undefined,
    };
  },
  
  validateProjectStructure: (project) => ({
    'has project ID': project.id !== undefined,
    'has project name': project.name && project.name.length > 0,
    'has valid status': ['Draft', 'RFP', 'Active', 'Completed', 'Cancelled'].includes(project.status),
    'has project number': project.number && project.number.id,
  }),
  
  validateResponseTime: (duration, threshold) => ({
    [`response under ${threshold}ms`]: duration < threshold,
  }),
};

export default {
  options,
  environments,
  getEnvironment,
  testCategories,
  getTestCategory,
  customChecks,
};