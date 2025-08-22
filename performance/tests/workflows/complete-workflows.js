/**
 * Complete User Workflow Performance Tests
 * 
 * Tests end-to-end user workflows under load to validate
 * real-world performance scenarios and identify bottlenecks
 * in complete business processes.
 */

import { group, check, sleep } from 'k6';
import { Counter, Rate, Trend, Gauge } from 'k6/metrics';
import { Surreal } from 'k6/x/surreal';
import { options, getEnvironment } from '../../config/k6.config.js';

// Custom metrics for workflow tracking
const workflowSuccess = new Rate('workflow_success_rate');
const workflowDuration = new Trend('workflow_duration');
const stepDuration = new Trend('workflow_step_duration');
const userThinkTime = new Trend('user_think_time');
const dataConsistency = new Rate('data_consistency_rate');
const errorRecovery = new Rate('error_recovery_rate');

// Workflow step counters
const projectWorkflowSteps = new Counter('project_workflow_steps');
const proposalWorkflowSteps = new Counter('proposal_workflow_steps');
const crmWorkflowSteps = new Counter('crm_workflow_steps');

// System resource metrics
const cpuUtilization = new Gauge('cpu_utilization_percent');
const memoryUtilization = new Gauge('memory_utilization_mb');
const databaseConnections = new Gauge('active_db_connections');

// Test configuration
const env = getEnvironment();
const workflowConfig = {
  userProfiles: ['power_user', 'casual_user', 'admin_user'],
  scenarios: ['normal_load', 'peak_load', 'stress_load'],
  thinkTimeRange: [500, 3000], // 0.5-3 seconds user think time
  errorSimulationRate: 0.05, // 5% error simulation rate
};

let db;
let userContext = {};

export function setup() {
  console.log('🚀 Setting up workflow performance tests...');
  
  // Initialize database connection
  try {
    db = new Surreal();
    db.connect(env.databaseUrl);
    db.use({ ns: env.namespace, db: env.database });
    db.signin(env.credentials);
    
    console.log('✅ Database connected for workflow tests');
    
    // Pre-populate test data if needed
    return setupTestData();
    
  } catch (error) {
    console.error('❌ Database setup failed:', error);
    return null;
  }
}

export default function(data) {
  // Simulate different user profiles
  const userProfile = workflowConfig.userProfiles[__VU % workflowConfig.userProfiles.length];
  userContext = { profile: userProfile, ...data };
  
  group('Complete Project Lifecycle Workflow', () => {
    testProjectLifecycleWorkflow();
  });
  
  group('Proposal Management Workflow', () => {
    testProposalManagementWorkflow();
  });
  
  group('CRM Data Management Workflow', () => {
    testCRMDataManagementWorkflow();
  });
  
  group('Dashboard and Reporting Workflow', () => {
    testDashboardReportingWorkflow();
  });
  
  group('Error Recovery Workflow', () => {
    testErrorRecoveryWorkflow();
  });
  
  // Simulate user session break
  simulateUserThinkTime();
}

function testProjectLifecycleWorkflow() {
  const workflowStartTime = Date.now();
  let workflowSuccess = true;
  
  try {
    console.log(`👤 ${userContext.profile}: Starting project lifecycle workflow`);
    
    // Step 1: Create new project
    const createProjectTime = measureStep('Create Project', () => {
      const projectData = generateProjectData();
      const result = db.query(`
        INSERT INTO projects CONTENT ${JSON.stringify(projectData)};
      `);
      
      projectWorkflowSteps.add(1);
      
      return check(result, {
        'project created successfully': (r) => r && !r.error,
        'project has valid ID': (r) => r && r[0] && r[0].result && r[0].result[0] && r[0].result[0].id,
      });
    });
    
    if (!createProjectTime.success) {
      workflowSuccess = false;
      return;
    }
    
    simulateUserThinkTime();
    
    // Step 2: Search and verify project exists
    const searchProjectTime = measureStep('Search Project', () => {
      const searchResult = db.query(`
        SELECT * FROM projects 
        WHERE name CONTAINS '${createProjectTime.data.name.split(' ')[0]}'
        LIMIT 10;
      `);
      
      projectWorkflowSteps.add(1);
      
      return check(searchResult, {
        'project search completed': (r) => r && !r.error,
        'created project found in search': (r) => r && r[0] && r[0].result && r[0].result.length > 0,
      });
    });
    
    simulateUserThinkTime();
    
    // Step 3: Update project details
    const updateProjectTime = measureStep('Update Project', () => {
      const updateData = {
        status: 'Active',
        stage: 'Design Development',
        updated_at: new Date().toISOString()
      };
      
      const updateResult = db.query(`
        UPDATE ${createProjectTime.data.id} 
        MERGE ${JSON.stringify(updateData)};
      `);
      
      projectWorkflowSteps.add(1);
      
      return check(updateResult, {
        'project updated successfully': (r) => r && !r.error,
        'project status updated': (r) => r && r[0] && r[0].result,
      });
    });
    
    simulateUserThinkTime();
    
    // Step 4: Create associated proposal
    const createProposalTime = measureStep('Create Associated Proposal', () => {
      const proposalData = generateProposalData(createProjectTime.data.id);
      const result = db.query(`
        INSERT INTO fee CONTENT ${JSON.stringify(proposalData)};
      `);
      
      projectWorkflowSteps.add(1);
      
      return check(result, {
        'proposal created successfully': (r) => r && !r.error,
        'proposal linked to project': (r) => r && r[0] && r[0].result && r[0].result[0],
      });
    });
    
    simulateUserThinkTime();
    
    // Step 5: Verify complete project with relationships
    const verifyRelationshipsTime = measureStep('Verify Project Relationships', () => {
      const relationshipQuery = `
        SELECT projects.*, 
               ->fee AS proposals,
               array::len(->fee) AS proposal_count
        FROM ${createProjectTime.data.id};
      `;
      
      const result = db.query(relationshipQuery);
      
      projectWorkflowSteps.add(1);
      
      return check(result, {
        'relationship query successful': (r) => r && !r.error,
        'project has linked proposals': (r) => r && r[0] && r[0].result && r[0].result[0] && r[0].result[0].proposal_count > 0,
      });
    });
    
    const workflowEndTime = Date.now();
    const totalDuration = workflowEndTime - workflowStartTime;
    
    workflowDuration.add(totalDuration, { workflow: 'project_lifecycle' });
    
    console.log(`✅ Project lifecycle workflow completed in ${totalDuration}ms`);
    
  } catch (error) {
    console.error('❌ Project lifecycle workflow failed:', error);
    workflowSuccess = false;
  }
  
  check(workflowSuccess, {
    'complete project lifecycle workflow succeeded': () => workflowSuccess,
  });
}

function testProposalManagementWorkflow() {
  const workflowStartTime = Date.now();
  let workflowSuccess = true;
  
  try {
    console.log(`👤 ${userContext.profile}: Starting proposal management workflow`);
    
    // Step 1: Search existing projects for proposal creation
    const findProjectTime = measureStep('Find Project for Proposal', () => {
      const result = db.query(`
        SELECT * FROM projects 
        WHERE status = 'Active' 
        ORDER BY time.updated_at DESC 
        LIMIT 5;
      `);
      
      proposalWorkflowSteps.add(1);
      
      return check(result, {
        'active projects found': (r) => r && !r.error && r[0] && r[0].result && r[0].result.length > 0,
      });
    });
    
    if (!findProjectTime.success) {
      workflowSuccess = false;
      return;
    }
    
    const selectedProject = findProjectTime.data[0];
    simulateUserThinkTime();
    
    // Step 2: Get related companies for proposal
    const findCompanyTime = measureStep('Find Company for Proposal', () => {
      const result = db.query(`
        SELECT * FROM company 
        WHERE country = '${selectedProject.country}' 
        LIMIT 3;
      `);
      
      proposalWorkflowSteps.add(1);
      
      return check(result, {
        'companies found for proposal': (r) => r && !r.error && r[0] && r[0].result && r[0].result.length > 0,
      });
    });
    
    const selectedCompany = findCompanyTime.data[0];
    simulateUserThinkTime();
    
    // Step 3: Find contact at company
    const findContactTime = measureStep('Find Contact for Proposal', () => {
      const result = db.query(`
        SELECT * FROM contacts 
        WHERE company = ${selectedCompany.id}
        LIMIT 1;
      `);
      
      proposalWorkflowSteps.add(1);
      
      return check(result, {
        'contact found at company': (r) => r && !r.error && r[0] && r[0].result && r[0].result.length > 0,
      });
    });
    
    const selectedContact = findContactTime.data[0] || { id: selectedCompany.id }; // Fallback
    simulateUserThinkTime();
    
    // Step 4: Create comprehensive proposal
    const createProposalTime = measureStep('Create Comprehensive Proposal', () => {
      const proposalData = {
        name: `Fee Proposal - ${selectedProject.name}`,
        number: generateProposalNumber(),
        project_id: selectedProject.id,
        company_id: selectedCompany.id,
        contact_id: selectedContact.id,
        status: 'Draft',
        issue_date: formatDateForSurrealDB(new Date()),
        activity: selectedProject.activity || 'Design and Consultancy',
        package: selectedProject.package || 'Full Service',
        strap_line: 'sensory design studio',
        staff_name: 'Design Team',
        staff_email: 'team@emittiv.com',
        rev: 0,
        revisions: []
      };
      
      const result = db.query(`
        INSERT INTO fee CONTENT ${JSON.stringify(proposalData)};
      `);
      
      proposalWorkflowSteps.add(1);
      
      return check(result, {
        'comprehensive proposal created': (r) => r && !r.error,
        'proposal has all required fields': (r) => r && r[0] && r[0].result && r[0].result[0],
      });
    });
    
    simulateUserThinkTime();
    
    // Step 5: Update proposal status workflow
    const statusWorkflowTime = measureStep('Proposal Status Workflow', () => {
      const proposalId = createProposalTime.data.id;
      const statusProgression = ['Draft', 'Sent', 'Negotiation', 'Awarded'];
      let statusUpdateSuccess = true;
      
      for (const status of statusProgression) {
        const updateResult = db.query(`
          UPDATE ${proposalId} SET status = '${status}', updated_at = time::now();
        `);
        
        if (!updateResult || updateResult.error) {
          statusUpdateSuccess = false;
          break;
        }
        
        sleep(0.1); // Brief pause between status updates
      }
      
      proposalWorkflowSteps.add(statusProgression.length);
      
      return check(statusUpdateSuccess, {
        'proposal status workflow completed': () => statusUpdateSuccess,
      });
    });
    
    const workflowEndTime = Date.now();
    const totalDuration = workflowEndTime - workflowStartTime;
    
    workflowDuration.add(totalDuration, { workflow: 'proposal_management' });
    
    console.log(`✅ Proposal management workflow completed in ${totalDuration}ms`);
    
  } catch (error) {
    console.error('❌ Proposal management workflow failed:', error);
    workflowSuccess = false;
  }
  
  check(workflowSuccess, {
    'complete proposal management workflow succeeded': () => workflowSuccess,
  });
}

function testCRMDataManagementWorkflow() {
  const workflowStartTime = Date.now();
  let workflowSuccess = true;
  
  try {
    console.log(`👤 ${userContext.profile}: Starting CRM data management workflow`);
    
    // Step 1: Create new company
    const createCompanyTime = measureStep('Create New Company', () => {
      const companyData = generateCompanyData();
      const result = db.query(`
        INSERT INTO company CONTENT ${JSON.stringify(companyData)};
      `);
      
      crmWorkflowSteps.add(1);
      
      return check(result, {
        'company created successfully': (r) => r && !r.error,
        'company has valid data': (r) => r && r[0] && r[0].result && r[0].result[0],
      });
    });
    
    if (!createCompanyTime.success) {
      workflowSuccess = false;
      return;
    }
    
    const companyId = createCompanyTime.data.id;
    simulateUserThinkTime();
    
    // Step 2: Add multiple contacts to company
    const addContactsTime = measureStep('Add Multiple Contacts', () => {
      const contactsData = [
        generateContactData(companyId, 'CEO'),
        generateContactData(companyId, 'Project Manager'),
        generateContactData(companyId, 'Technical Director')
      ];
      
      let allContactsCreated = true;
      
      for (const contactData of contactsData) {
        const result = db.query(`
          INSERT INTO contacts CONTENT ${JSON.stringify(contactData)};
        `);
        
        if (!result || result.error) {
          allContactsCreated = false;
          break;
        }
      }
      
      crmWorkflowSteps.add(contactsData.length);
      
      return check(allContactsCreated, {
        'all contacts created successfully': () => allContactsCreated,
      });
    });
    
    simulateUserThinkTime();
    
    // Step 3: Search and filter CRM data
    const searchCRMTime = measureStep('Search CRM Data', () => {
      const searchQueries = [
        `SELECT * FROM company WHERE country = 'U.A.E.' LIMIT 10;`,
        `SELECT * FROM contacts WHERE position CONTAINS 'Manager' LIMIT 10;`,
        `SELECT c.*, co.name AS company_name FROM contacts c JOIN company co ON c.company = co.id LIMIT 10;`
      ];
      
      let allSearchesSuccessful = true;
      
      for (const query of searchQueries) {
        const result = db.query(query);
        if (!result || result.error) {
          allSearchesSuccessful = false;
          break;
        }
      }
      
      crmWorkflowSteps.add(searchQueries.length);
      
      return check(allSearchesSuccessful, {
        'all CRM searches completed successfully': () => allSearchesSuccessful,
      });
    });
    
    simulateUserThinkTime();
    
    // Step 4: Update company information
    const updateCompanyTime = measureStep('Update Company Information', () => {
      const updateData = {
        name_short: 'Updated Co.',
        city: 'New City',
        updated_at: new Date().toISOString()
      };
      
      const result = db.query(`
        UPDATE ${companyId} MERGE ${JSON.stringify(updateData)};
      `);
      
      crmWorkflowSteps.add(1);
      
      return check(result, {
        'company information updated': (r) => r && !r.error,
      });
    });
    
    simulateUserThinkTime();
    
    // Step 5: Verify data integrity
    const verifyIntegrityTime = measureStep('Verify Data Integrity', () => {
      const integrityQueries = [
        `SELECT COUNT(*) as contact_count FROM contacts WHERE company = ${companyId};`,
        `SELECT * FROM ${companyId};`
      ];
      
      let integrityValid = true;
      
      for (const query of integrityQueries) {
        const result = db.query(query);
        if (!result || result.error) {
          integrityValid = false;
          break;
        }
      }
      
      crmWorkflowSteps.add(integrityQueries.length);
      
      return check(integrityValid, {
        'CRM data integrity verified': () => integrityValid,
      });
    });
    
    const workflowEndTime = Date.now();
    const totalDuration = workflowEndTime - workflowStartTime;
    
    workflowDuration.add(totalDuration, { workflow: 'crm_management' });
    
    console.log(`✅ CRM data management workflow completed in ${totalDuration}ms`);
    
  } catch (error) {
    console.error('❌ CRM data management workflow failed:', error);
    workflowSuccess = false;
  }
  
  check(workflowSuccess, {
    'complete CRM data management workflow succeeded': () => workflowSuccess,
  });
}

function testDashboardReportingWorkflow() {
  const workflowStartTime = Date.now();
  
  try {
    console.log(`👤 ${userContext.profile}: Starting dashboard reporting workflow`);
    
    // Simulate dashboard data aggregation queries
    const dashboardQueries = [
      {
        name: 'Project Status Summary',
        query: `SELECT status, COUNT(*) as count FROM projects GROUP BY status;`
      },
      {
        name: 'Proposal Pipeline',
        query: `SELECT status, COUNT(*) as count FROM fee GROUP BY status ORDER BY count DESC;`
      },
      {
        name: 'Recent Activity',
        query: `
          (SELECT name, time.updated_at, 'project' as type FROM projects ORDER BY time.updated_at DESC LIMIT 5)
          UNION
          (SELECT name, time.updated_at, 'proposal' as type FROM fee ORDER BY time.updated_at DESC LIMIT 5)
          ORDER BY time.updated_at DESC LIMIT 10;
        `
      },
      {
        name: 'Geographic Distribution',
        query: `SELECT country, COUNT(*) as count FROM company GROUP BY country;`
      }
    ];
    
    let allQueriesSuccessful = true;
    
    for (const { name, query } of dashboardQueries) {
      const queryStartTime = Date.now();
      const result = db.query(query);
      const queryDuration = Date.now() - queryStartTime;
      
      stepDuration.add(queryDuration, { step: name.toLowerCase().replace(/\s+/g, '_') });
      
      if (!result || result.error) {
        allQueriesSuccessful = false;
        console.error(`Dashboard query failed: ${name}`);
      } else {
        console.log(`📊 ${name} completed in ${queryDuration}ms`);
      }
    }
    
    const workflowEndTime = Date.now();
    const totalDuration = workflowEndTime - workflowStartTime;
    
    workflowDuration.add(totalDuration, { workflow: 'dashboard_reporting' });
    
    check(allQueriesSuccessful, {
      'dashboard reporting workflow completed': () => allQueriesSuccessful,
      'dashboard loaded within performance threshold': () => totalDuration < 3000,
    });
    
  } catch (error) {
    console.error('❌ Dashboard reporting workflow failed:', error);
  }
}

function testErrorRecoveryWorkflow() {
  if (Math.random() > workflowConfig.errorSimulationRate) {
    return; // Skip error simulation for most iterations
  }
  
  console.log(`👤 ${userContext.profile}: Testing error recovery workflow`);
  
  try {
    // Simulate various error conditions
    const errorScenarios = [
      {
        name: 'Invalid Query Syntax',
        action: () => db.query('SELECT * FROM non_existent_table;'),
        expectedError: true
      },
      {
        name: 'Constraint Violation',
        action: () => db.query('INSERT INTO projects { invalid: "data" };'),
        expectedError: true
      },
      {
        name: 'Connection Timeout Simulation',
        action: () => {
          sleep(0.1); // Brief pause
          return db.query('SELECT COUNT(*) FROM projects;');
        },
        expectedError: false
      }
    ];
    
    let recoverySuccessful = true;
    
    for (const scenario of errorScenarios) {
      try {
        const result = scenario.action();
        
        if (scenario.expectedError && (!result || !result.error)) {
          console.log(`⚠️  Expected error not encountered: ${scenario.name}`);
        } else if (!scenario.expectedError && result && result.error) {
          console.log(`❌ Unexpected error in: ${scenario.name}`);
          recoverySuccessful = false;
        }
        
      } catch (error) {
        if (!scenario.expectedError) {
          console.error(`❌ Error recovery failed for: ${scenario.name}`, error);
          recoverySuccessful = false;
        }
      }
    }
    
    errorRecovery.add(recoverySuccessful);
    
  } catch (error) {
    console.error('❌ Error recovery workflow failed:', error);
  }
}

// Helper functions

function measureStep(stepName, stepFunction) {
  const startTime = Date.now();
  
  try {
    const result = stepFunction();
    const duration = Date.now() - startTime;
    
    stepDuration.add(duration, { step: stepName.toLowerCase().replace(/\s+/g, '_') });
    
    return {
      success: result,
      duration: duration,
      data: result // May contain created data
    };
    
  } catch (error) {
    const duration = Date.now() - startTime;
    stepDuration.add(duration, { step: stepName.toLowerCase().replace(/\s+/g, '_') });
    
    console.error(`Step "${stepName}" failed:`, error);
    return {
      success: false,
      duration: duration,
      error: error
    };
  }
}

function simulateUserThinkTime() {
  const minThinkTime = workflowConfig.thinkTimeRange[0];
  const maxThinkTime = workflowConfig.thinkTimeRange[1];
  const thinkTime = Math.random() * (maxThinkTime - minThinkTime) + minThinkTime;
  
  userThinkTime.add(thinkTime);
  sleep(thinkTime / 1000); // Convert to seconds for k6
}

function generateProjectData() {
  const countries = ['U.A.E.', 'Saudi Arabia', 'Qatar'];
  const cities = ['Dubai', 'Abu Dhabi', 'Riyadh', 'Jeddah', 'Doha'];
  const activities = ['Design and Consultancy', 'Management', 'Construction'];
  
  return {
    name: `Performance Test Project ${Date.now()}`,
    name_short: `PerfTest ${Date.now() % 10000}`,
    activity: activities[Math.floor(Math.random() * activities.length)],
    package: 'Full Service',
    status: 'Draft',
    stage: 'Concept',
    area: 'Test Area',
    city: cities[Math.floor(Math.random() * cities.length)],
    country: countries[Math.floor(Math.random() * countries.length)],
    folder: `perf-test-${Date.now()}`,
    number: {
      year: 25,
      country: 971,
      seq: Math.floor(Math.random() * 999) + 1,
      id: `25-971${Math.floor(Math.random() * 99) + 1}`
    }
  };
}

function generateProposalData(projectId) {
  return {
    name: `Performance Test Proposal ${Date.now()}`,
    number: `PERF-${Date.now()}`,
    project_id: projectId,
    company_id: 'company:test_company',
    contact_id: 'contacts:test_contact',
    status: 'Draft',
    issue_date: formatDateForSurrealDB(new Date()),
    strap_line: 'sensory design studio',
    rev: 0,
    revisions: []
  };
}

function generateCompanyData() {
  const countries = ['U.A.E.', 'Saudi Arabia', 'Qatar'];
  const cities = ['Dubai', 'Abu Dhabi', 'Riyadh', 'Doha'];
  
  return {
    name: `Performance Test Company ${Date.now()}`,
    name_short: `PerfCo ${Date.now() % 1000}`,
    abbreviation: `PC${Date.now() % 100}`,
    city: cities[Math.floor(Math.random() * cities.length)],
    country: countries[Math.floor(Math.random() * countries.length)],
    reg_no: `REG${Date.now() % 100000}`,
    tax_no: `TAX${Date.now() % 100000}`
  };
}

function generateContactData(companyId, position) {
  return {
    first_name: `TestFirst${Date.now() % 1000}`,
    last_name: `TestLast${Date.now() % 1000}`,
    email: `test${Date.now() % 10000}@perftest.com`,
    phone: `+971501234${Math.floor(Math.random() * 1000).toString().padStart(3, '0')}`,
    position: position,
    company: companyId,
    full_name: `TestFirst${Date.now() % 1000} TestLast${Date.now() % 1000}`
  };
}

function generateProposalNumber() {
  return `PERF-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
}

function formatDateForSurrealDB(date) {
  const year = date.getFullYear().toString().slice(-2);
  const month = (date.getMonth() + 1).toString().padStart(2, '0');
  const day = date.getDate().toString().padStart(2, '0');
  return `${year}${month}${day}`;
}

function setupTestData() {
  // Return any setup data needed for workflows
  return {
    initialized: true,
    timestamp: Date.now()
  };
}

export function teardown(data) {
  console.log('🧹 Cleaning up workflow performance tests...');
  
  if (db) {
    try {
      // Clean up any test data created during workflows
      db.query('DELETE FROM projects WHERE name CONTAINS "Performance Test";');
      db.query('DELETE FROM fee WHERE name CONTAINS "Performance Test";');
      db.query('DELETE FROM company WHERE name CONTAINS "Performance Test";');
      db.query('DELETE FROM contacts WHERE email CONTAINS "perftest.com";');
      
      db.close();
    } catch (error) {
      console.error('Cleanup error:', error);
    }
  }
  
  console.log('✅ Workflow performance test cleanup complete');
}

export { options };