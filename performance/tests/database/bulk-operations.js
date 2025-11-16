/**
 * Database Bulk Operations Performance Tests
 * 
 * Tests the performance of bulk CRUD operations on SurrealDB
 * to validate database performance at scale.
 */

import { check, group, sleep } from 'k6';
import { Counter, Rate, Trend, Gauge } from 'k6/metrics';
import { Surreal } from 'k6/x/surreal'; // Using k6-surreal extension
import { options, getEnvironment } from '../../config/k6.config.js';

// Custom metrics for database operations
const bulkInsertRate = new Rate('bulk_insert_success_rate');
const bulkInsertDuration = new Trend('bulk_insert_duration');
const bulkUpdateRate = new Rate('bulk_update_success_rate');
const bulkUpdateDuration = new Trend('bulk_update_duration');
const bulkDeleteRate = new Rate('bulk_delete_success_rate');
const bulkDeleteDuration = new Trend('bulk_delete_duration');
const queryDuration = new Trend('query_duration');
const connectionDuration = new Trend('connection_duration');
const memoryUsage = new Gauge('memory_usage_mb');
const activeConnections = new Gauge('active_connections');

// Test configuration
const env = getEnvironment();
const testConfig = {
  batchSizes: [10, 50, 100, 500, 1000],
  maxBatchSize: parseInt(__ENV.MAX_BATCH_SIZE || '1000'),
  testIterations: parseInt(__ENV.TEST_ITERATIONS || '10'),
  warmupIterations: parseInt(__ENV.WARMUP_ITERATIONS || '2'),
};

let db;
let testData = {
  companies: [],
  contacts: [],
  projects: [],
  proposals: []
};

// Setup function - runs once per VU
export function setup() {
  console.log('🚀 Setting up bulk operations performance test...');
  
  // Pre-generate test data for consistent testing
  const setup_data = generateTestData();
  
  console.log(`📊 Generated test data: ${Object.keys(setup_data).map(k => `${k}=${setup_data[k].length}`).join(', ')}`);
  
  return setup_data;
}

// Main test function
export default function(data) {
  testData = data;
  
  group('Database Connection Performance', () => {
    testDatabaseConnection();
  });
  
  group('Bulk Insert Performance', () => {
    testBulkInserts();
  });
  
  group('Bulk Update Performance', () => {
    testBulkUpdates();
  });
  
  group('Bulk Delete Performance', () => {
    testBulkDeletes();
  });
  
  group('Complex Query Performance', () => {
    testComplexQueries();
  });
  
  group('Connection Pool Stress', () => {
    testConnectionPooling();
  });
  
  // Brief pause between iterations
  sleep(1);
}

function testDatabaseConnection() {
  const startTime = Date.now();
  
  try {
    db = new Surreal();
    db.connect(env.databaseUrl);
    db.use({ ns: env.namespace, db: env.database });
    db.signin(env.credentials);
    
    const duration = Date.now() - startTime;
    connectionDuration.add(duration);
    
    check(db, {
      'database connected successfully': (db) => db !== null,
    });
    
    activeConnections.add(1);
    
  } catch (error) {
    console.error('Database connection failed:', error);
    check(false, { 'database connection failed': false });
  }
}

function testBulkInserts() {
  if (!db) return;
  
  // Test different batch sizes
  for (const batchSize of testConfig.batchSizes) {
    if (batchSize > testConfig.maxBatchSize) continue;
    
    group(`Bulk Insert - ${batchSize} records`, () => {
      // Test company inserts
      const companies = testData.companies.slice(0, batchSize);
      const startTime = Date.now();
      
      try {
        // Use SurrealDB's bulk insert capability
        const result = db.query(`
          INSERT INTO company ${JSON.stringify(companies)};
        `);
        
        const duration = Date.now() - startTime;
        bulkInsertDuration.add(duration);
        
        const success = check(result, {
          [`bulk insert ${batchSize} companies succeeded`]: (r) => r && !r.error,
          [`bulk insert ${batchSize} completed in reasonable time`]: () => duration < 5000,
        });
        
        bulkInsertRate.add(success);
        
        // Calculate throughput
        const throughput = batchSize / (duration / 1000);
        console.log(`📈 Bulk insert throughput: ${throughput.toFixed(2)} records/second`);
        
      } catch (error) {
        console.error(`Bulk insert failed for ${batchSize} records:`, error);
        bulkInsertRate.add(false);
      }
    });
  }
}

function testBulkUpdates() {
  if (!db) return;
  
  // First, ensure we have data to update
  const setupResult = db.query('SELECT id FROM company LIMIT 1000');
  const existingRecords = setupResult[0]?.result || [];
  
  if (existingRecords.length === 0) {
    console.log('⚠️  No existing records for bulk update test');
    return;
  }
  
  for (const batchSize of testConfig.batchSizes) {
    if (batchSize > existingRecords.length) continue;
    
    group(`Bulk Update - ${batchSize} records`, () => {
      const recordsToUpdate = existingRecords.slice(0, batchSize);
      const startTime = Date.now();
      
      try {
        // Generate update queries
        const updateQueries = recordsToUpdate.map(record => 
          `UPDATE ${record.id} SET updated_at = time::now(), name_short = "Updated ${Date.now()}";`
        ).join('\n');
        
        const result = db.query(updateQueries);
        
        const duration = Date.now() - startTime;
        bulkUpdateDuration.add(duration);
        
        const success = check(result, {
          [`bulk update ${batchSize} records succeeded`]: (r) => r && !r.error,
          [`bulk update ${batchSize} completed in reasonable time`]: () => duration < 10000,
        });
        
        bulkUpdateRate.add(success);
        
        // Calculate throughput
        const throughput = batchSize / (duration / 1000);
        console.log(`📊 Bulk update throughput: ${throughput.toFixed(2)} records/second`);
        
      } catch (error) {
        console.error(`Bulk update failed for ${batchSize} records:`, error);
        bulkUpdateRate.add(false);
      }
    });
  }
}

function testBulkDeletes() {
  if (!db) return;
  
  // Create temporary records for deletion testing
  const tempRecords = generateTempCompanies(100);
  
  try {
    // Insert temp records first
    db.query(`INSERT INTO company ${JSON.stringify(tempRecords)}`);
    
    for (const batchSize of [10, 25, 50]) { // Smaller batches for delete
      group(`Bulk Delete - ${batchSize} records`, () => {
        const startTime = Date.now();
        
        try {
          // Delete records by pattern
          const result = db.query(`
            DELETE FROM company 
            WHERE name CONTAINS "TEMP_DELETE_TEST" 
            LIMIT ${batchSize};
          `);
          
          const duration = Date.now() - startTime;
          bulkDeleteDuration.add(duration);
          
          const success = check(result, {
            [`bulk delete ${batchSize} records succeeded`]: (r) => r && !r.error,
            [`bulk delete ${batchSize} completed in reasonable time`]: () => duration < 5000,
          });
          
          bulkDeleteRate.add(success);
          
        } catch (error) {
          console.error(`Bulk delete failed for ${batchSize} records:`, error);
          bulkDeleteRate.add(false);
        }
      });
    }
    
  } catch (error) {
    console.error('Failed to set up bulk delete test:', error);
  }
}

function testComplexQueries() {
  if (!db) return;
  
  const complexQueries = [
    {
      name: 'Projects with Proposals Join',
      query: `
        SELECT projects.*, 
               array::len(->fee) AS proposal_count,
               ->fee[WHERE status = 'Sent'] AS active_proposals
        FROM projects 
        WHERE status = 'Active'
        LIMIT 100;
      `
    },
    {
      name: 'Company Activity Aggregation',
      query: `
        SELECT company.country,
               COUNT(*) AS company_count,
               COUNT(->contacts) AS contact_count,
               COUNT(<-fee.company_id) AS proposal_count
        FROM company
        GROUP BY company.country
        ORDER BY company_count DESC;
      `
    },
    {
      name: 'Full-text Search Across Tables',
      query: `
        (SELECT *, 'project' AS type FROM projects WHERE name CONTAINS 'Tower')
        UNION
        (SELECT *, 'company' AS type FROM company WHERE name CONTAINS 'Tower')
        UNION
        (SELECT *, 'proposal' AS type FROM fee WHERE name CONTAINS 'Tower')
        LIMIT 50;
      `
    }
  ];
  
  complexQueries.forEach(({ name, query }) => {
    group(name, () => {
      const startTime = Date.now();
      
      try {
        const result = db.query(query);
        const duration = Date.now() - startTime;
        
        queryDuration.add(duration, { query_type: name.toLowerCase().replace(/\s+/g, '_') });
        
        check(result, {
          [`${name} query succeeded`]: (r) => r && !r.error,
          [`${name} query completed in reasonable time`]: () => duration < 3000,
          [`${name} query returned results`]: (r) => r && r[0] && r[0].result && r[0].result.length > 0,
        });
        
      } catch (error) {
        console.error(`Complex query "${name}" failed:`, error);
      }
    });
  });
}

function testConnectionPooling() {
  if (!db) return;
  
  // Simulate multiple concurrent connections
  const connections = [];
  const maxConnections = 10;
  
  group('Connection Pool Stress Test', () => {
    const startTime = Date.now();
    
    try {
      // Create multiple connections
      for (let i = 0; i < maxConnections; i++) {
        const conn = new Surreal();
        conn.connect(env.databaseUrl);
        conn.use({ ns: env.namespace, db: env.database });
        conn.signin(env.credentials);
        connections.push(conn);
      }
      
      // Perform queries on all connections simultaneously
      const promises = connections.map((conn, index) => {
        return conn.query(`SELECT COUNT(*) as count FROM company WHERE id CONTAINS '${index}';`);
      });
      
      // Wait for all queries to complete (simulated with sleep in k6)
      sleep(2);
      
      const duration = Date.now() - startTime;
      
      check(connections, {
        'all connections created successfully': (conns) => conns.length === maxConnections,
        'connection pool test completed': () => duration < 10000,
      });
      
      activeConnections.add(connections.length);
      
      // Cleanup connections
      connections.forEach(conn => {
        try {
          conn.close();
        } catch (e) {
          // Ignore close errors
        }
      });
      
    } catch (error) {
      console.error('Connection pooling test failed:', error);
    }
  });
}

// Test data generation functions
function generateTestData() {
  return {
    companies: generateTempCompanies(1000),
    contacts: [], // Could be expanded
    projects: [], // Could be expanded
    proposals: [] // Could be expanded
  };
}

function generateTempCompanies(count) {
  const companies = [];
  const countries = ['U.A.E.', 'Saudi Arabia', 'Qatar'];
  const cities = {
    'U.A.E.': ['Dubai', 'Abu Dhabi', 'Sharjah'],
    'Saudi Arabia': ['Riyadh', 'Jeddah', 'Dammam'],
    'Qatar': ['Doha', 'Al Rayyan']
  };
  
  for (let i = 0; i < count; i++) {
    const country = countries[i % countries.length];
    const city = cities[country][i % cities[country].length];
    
    companies.push({
      name: `TEMP_DELETE_TEST_Company_${i}`,
      name_short: `TDT${i}`,
      abbreviation: `TDT${i}`,
      city: city,
      country: country,
      reg_no: `REG${i.toString().padStart(6, '0')}`,
      tax_no: `TAX${i.toString().padStart(8, '0')}`
    });
  }
  
  return companies;
}

// Teardown function - runs once at end
export function teardown(data) {
  console.log('🧹 Cleaning up bulk operations test...');
  
  if (db) {
    try {
      // Clean up any remaining test data
      db.query('DELETE FROM company WHERE name CONTAINS "TEMP_DELETE_TEST";');
      db.close();
    } catch (error) {
      console.error('Cleanup error:', error);
    }
  }
  
  console.log('✅ Bulk operations test cleanup complete');
}

export { options };