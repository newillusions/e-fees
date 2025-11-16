/**
 * SurrealDB Performance Testing Configuration
 * 
 * Configuration for database-specific performance testing including
 * connection settings, query optimization, and data generation parameters.
 */

export const databaseConfig = {
  connection: {
    // Primary database configuration
    primary: {
      url: process.env.SURREALDB_URL || 'ws://10.0.1.17:8000',
      namespace: process.env.SURREALDB_NS || 'emittiv',
      database: process.env.SURREALDB_DB || 'projects',
      auth: {
        username: process.env.SURREALDB_USER || 'martin',
        password: process.env.SURREALDB_PASS || 'default_password'
      }
    },
    
    // Test database configuration (for destructive operations)
    test: {
      url: process.env.SURREALDB_TEST_URL || 'ws://localhost:8000',
      namespace: 'emittiv_perf_test',
      database: 'projects_perf_test',
      auth: {
        username: 'perf_test_user',
        password: process.env.SURREALDB_TEST_PASS || 'test_password'
      }
    },

    // Connection pool settings
    pool: {
      maxConnections: 20,
      connectionTimeout: 5000, // 5 seconds
      idleTimeout: 30000, // 30 seconds
      retryAttempts: 3,
      retryDelay: 1000, // 1 second
    }
  },

  // Performance test data scaling configurations
  dataScaling: {
    // Current production baseline (as of dataset analysis)
    production: {
      projects: 48,
      proposals: 37, // fee table
      companies: 19,
      contacts: 50, // estimated
      countries: 250, // reference data
      currencies: 180, // reference data
    },

    // Small scale testing (5-10x production)
    small: {
      projects: 500,
      proposals: 1000,
      companies: 100,
      contacts: 500,
      // Reference data remains the same
      countries: 250,
      currencies: 180,
    },

    // Medium scale testing (50-100x production)
    medium: {
      projects: 2500,
      proposals: 5000,
      companies: 500,
      contacts: 2500,
      countries: 250,
      currencies: 180,
    },

    // Large scale testing (200-500x production)
    large: {
      projects: 10000,
      proposals: 25000,
      companies: 2000,
      contacts: 10000,
      countries: 250,
      currencies: 180,
    },

    // Extreme scale testing (1000x+ production)
    extreme: {
      projects: 50000,
      proposals: 125000,
      companies: 10000,
      contacts: 50000,
      countries: 250,
      currencies: 180,
    }
  },

  // Query performance benchmarks
  queryBenchmarks: {
    // Single record operations (in milliseconds)
    singleRecord: {
      select: 5,    // SELECT * FROM table WHERE id = X
      insert: 10,   // INSERT INTO table ...
      update: 15,   // UPDATE table SET ... WHERE id = X
      delete: 10,   // DELETE FROM table WHERE id = X
    },

    // List operations (in milliseconds)
    listOperations: {
      small: 25,    // < 100 records
      medium: 100,  // 100-1000 records  
      large: 500,   // 1000-10000 records
      xlarge: 2000, // 10000+ records
    },

    // Complex query operations (in milliseconds)
    complexQueries: {
      simpleJoin: 50,        // 2 table join
      multipleJoin: 200,     // 3+ table join
      aggregation: 100,      // COUNT, SUM, AVG queries
      fullTextSearch: 300,   // Text search across multiple fields
      rangeQuery: 150,       // Date/number range queries
    },

    // Bulk operations (operations per second)
    bulkOperations: {
      bulkInsert: 1000,      // Records per second
      bulkUpdate: 500,       // Records per second
      bulkDelete: 800,       // Records per second
    }
  },

  // Test queries for performance validation
  testQueries: {
    // Basic CRUD operations
    crud: [
      // Projects
      "SELECT * FROM projects LIMIT 1000",
      "SELECT * FROM projects WHERE status = 'Active'",
      "SELECT * FROM projects WHERE time.created_at > time::now() - 30d",
      
      // Proposals (fee table)
      "SELECT * FROM fee LIMIT 1000",
      "SELECT * FROM fee WHERE status = 'Sent'",
      "SELECT * FROM fee WHERE issue_date CONTAINS '25'", // Current year
      
      // Companies
      "SELECT * FROM company WHERE country = 'U.A.E.'",
      "SELECT * FROM company ORDER BY time.updated_at DESC LIMIT 100",
      
      // Contacts
      "SELECT * FROM contacts WHERE company != NONE LIMIT 1000",
      "SELECT * FROM contacts WHERE email CONTAINS '@emittiv.com'",
    ],

    // Complex join queries (realistic application queries)
    joins: [
      // Project with related data
      `SELECT projects.*, 
              company.name AS company_name,
              contacts.full_name AS contact_name,
              fee.status AS fee_status
       FROM projects 
       LEFT JOIN fee ON fee.project_id = projects.id
       LEFT JOIN company ON fee.company_id = company.id
       LEFT JOIN contacts ON fee.contact_id = contacts.id
       LIMIT 500`,

      // Fee proposals with full context
      `SELECT fee.*,
              projects.name AS project_name,
              projects.number.id AS project_number,
              company.name AS company_name,
              contacts.full_name AS contact_name
       FROM fee
       JOIN projects ON fee.project_id = projects.id
       JOIN company ON fee.company_id = company.id  
       JOIN contacts ON fee.contact_id = contacts.id
       WHERE fee.status IN ['Sent', 'Negotiation']
       ORDER BY fee.time.updated_at DESC
       LIMIT 200`,

      // Dashboard aggregations
      `SELECT projects.status, 
              COUNT(*) AS count,
              AVG(array::len(->fee)) AS avg_proposals
       FROM projects
       GROUP BY projects.status`,
    ],

    // Search and filtering queries
    search: [
      // Text search across projects
      "SELECT * FROM projects WHERE name CONTAINS $search_term OR area CONTAINS $search_term",
      
      // Multi-field company search  
      "SELECT * FROM company WHERE name CONTAINS $search_term OR city CONTAINS $search_term OR country CONTAINS $search_term",
      
      // Contact search with company join
      `SELECT contacts.*, company.name AS company_name
       FROM contacts
       JOIN company ON contacts.company = company.id
       WHERE contacts.full_name CONTAINS $search_term
          OR contacts.email CONTAINS $search_term  
          OR company.name CONTAINS $search_term`,
    ],

    // Performance stress queries
    stress: [
      // Large result set
      "SELECT * FROM projects ORDER BY time.created_at DESC LIMIT 5000",
      
      // Complex aggregation
      `SELECT 
         projects.country,
         projects.status,
         COUNT(*) AS project_count,
         COUNT(->fee) AS proposal_count,
         AVG(->fee[WHERE status = 'Awarded']) AS success_rate
       FROM projects
       GROUP BY projects.country, projects.status
       ORDER BY project_count DESC`,
       
      // Expensive join with sorting
      `SELECT fee.*, projects.name, company.name, contacts.full_name
       FROM fee
       JOIN projects ON fee.project_id = projects.id
       JOIN company ON fee.company_id = company.id
       JOIN contacts ON fee.contact_id = contacts.id
       ORDER BY fee.time.updated_at DESC, projects.name ASC
       LIMIT 1000`,
    ]
  },

  // Index optimization testing
  indexOptimization: {
    // Indexes that should exist for optimal performance
    requiredIndexes: [
      // Projects
      'project_number_unique',
      'projects_status_index',
      'projects_country_index',
      'projects_created_at_index',
      
      // Fee proposals
      'fee_project_id_index',
      'fee_company_id_index', 
      'fee_contact_id_index',
      'fee_status_index',
      'fee_issue_date_index',
      
      // Companies
      'company_country_index',
      'company_name_index',
      
      // Contacts
      'contacts_company_index',
      'contacts_email_index',
    ],

    // Query performance with/without indexes
    indexImpactTests: [
      {
        name: 'Project status filtering',
        query: "SELECT * FROM projects WHERE status = 'Active'",
        withIndex: 'projects_status_index',
        expectedImprovement: '10x', // 10x faster with index
      },
      {
        name: 'Fee proposal by project',
        query: "SELECT * FROM fee WHERE project_id = $project_id",
        withIndex: 'fee_project_id_index',
        expectedImprovement: '20x',
      },
      {
        name: 'Company search by name',
        query: "SELECT * FROM company WHERE name CONTAINS $search_term",
        withIndex: 'company_name_index',
        expectedImprovement: '5x',
      }
    ]
  },

  // Memory and connection testing
  connectionTesting: {
    // Connection pool stress testing
    maxConcurrentConnections: 50,
    connectionHoldTime: 30000, // 30 seconds
    
    // WebSocket stability testing
    websocketTests: {
      reconnectionAttempts: 10,
      reconnectionDelay: 1000,
      heartbeatInterval: 30000,
      maxIdleTime: 300000, // 5 minutes
    },

    // Memory usage monitoring
    memoryThresholds: {
      connectionMemoryMB: 5,    // Per connection
      queryResultMemoryMB: 50,  // Per large query result
      totalMemoryMB: 200,       // Total DB connection memory
    }
  },

  // Data generation parameters for realistic testing
  dataGeneration: {
    // Realistic data distributions (based on current production data)
    distributions: {
      projectStatus: {
        'Active': 0.4,      // 40% active projects
        'Completed': 0.3,   // 30% completed
        'RFP': 0.15,        // 15% in RFP stage
        'Cancelled': 0.1,   // 10% cancelled
        'Draft': 0.05,      // 5% drafts
      },
      
      feeStatus: {
        'Sent': 0.35,       // 35% sent proposals
        'Draft': 0.25,      // 25% drafts
        'Negotiation': 0.15, // 15% in negotiation
        'Awarded': 0.1,     // 10% awarded
        'Lost': 0.1,        // 10% lost
        'Completed': 0.05,  // 5% completed
      },

      countries: {
        'U.A.E.': 0.6,      // 60% UAE projects
        'Saudi Arabia': 0.3, // 30% Saudi projects
        'Other': 0.1,       // 10% other countries
      }
    },

    // Realistic data patterns
    patterns: {
      projectsPerYear: [5, 15, 25], // Min, average, max projects per year
      proposalsPerProject: [0.5, 1.2, 3], // Min, avg, max proposals per project
      contactsPerCompany: [1, 2.5, 8], // Min, avg, max contacts per company
    }
  }
};

export default databaseConfig;