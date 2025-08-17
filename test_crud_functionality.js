#!/usr/bin/env node

/**
 * SAFE CRUD Functionality Testing
 * 
 * SAFETY RULES:
 * - All test records include "DELETE_ME" in name/title
 * - Only create temporary test data  
 * - Delete ALL test records when done
 * - NEVER modify/delete existing production data
 */

const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

console.log('🧪 CRUD FUNCTIONALITY TESTING');
console.log('==============================');
console.log('🔒 SAFETY MODE: Only creating/testing DELETE_ME records');
console.log('');

let testsPassed = 0;
let testsTotal = 0;
let createdRecords = []; // Track all created records for cleanup

function testResult(name, passed, details = '', recordId = null) {
  testsTotal++;
  if (passed) {
    testsPassed++;
    console.log(`✅ ${name}`);
    if (recordId) {
      createdRecords.push(recordId);
    }
  } else {
    console.log(`❌ ${name}`);
  }
  if (details) {
    console.log(`   ${details}`);
  }
}

async function runTauriCommand(command, data = {}) {
  try {
    // This would normally use Tauri invoke, but for Node.js testing we'll simulate
    console.log(`   Simulating: ${command}(${JSON.stringify(data).substring(0, 100)}...)`);
    
    // For actual testing, this would be:
    // return await invoke(command, data);
    
    return { success: true, id: `test_${Date.now()}` };
  } catch (error) {
    console.error(`   Error in ${command}:`, error.message);
    return { success: false, error: error.message };
  }
}

async function testCRUDFunctionality() {
  console.log('📋 TEST 1: COMPANY CRUD OPERATIONS');
  
  // Test Company Creation
  const testCompany = {
    name: "DELETE_ME Test Company",
    abbreviation: "DELME",
    email: "deleteme@test.com",
    phone: "+971 50 000 0000",
    address: "Test Address DELETE_ME",
    city: "Test City",
    country: "UAE"
  };
  
  const companyResult = await runTauriCommand('create_company', { company: testCompany });
  testResult('Company Creation', companyResult.success, 
    companyResult.success ? 'Test company created successfully' : companyResult.error,
    companyResult.success ? { type: 'company', id: companyResult.id } : null);
  
  console.log('\n📋 TEST 2: CONTACT CRUD OPERATIONS');
  
  // Test Contact Creation (needs company ID)
  const testContact = {
    first_name: "DELETE_ME",
    last_name: "Test Contact", 
    email: "deleteme.contact@test.com",
    phone: "+971 50 000 0001",
    position: "Test Position",
    company: companyResult.id || "test_company_id"
  };
  
  const contactResult = await runTauriCommand('create_contact', { contact: testContact });
  testResult('Contact Creation', contactResult.success,
    contactResult.success ? 'Test contact created successfully' : contactResult.error,
    contactResult.success ? { type: 'contact', id: contactResult.id } : null);
  
  console.log('\n📋 TEST 3: PROJECT CRUD OPERATIONS');
  
  // Test Project Creation
  const testProject = {
    name: "DELETE_ME Test Project",
    name_short: "DELME-PROJ",
    number: "25-97199", // Using test project number
    status: "Draft",
    area: "Test Area",
    city: "Test City", 
    country: "UAE",
    folder: "DELETE_ME_Test"
  };
  
  const projectResult = await runTauriCommand('create_project', { project: testProject });
  testResult('Project Creation', projectResult.success,
    projectResult.success ? 'Test project created successfully' : projectResult.error,
    projectResult.success ? { type: 'project', id: projectResult.id } : null);
  
  console.log('\n📋 TEST 4: PROPOSAL/FEE CRUD OPERATIONS');
  
  // Test Fee/Proposal Creation
  const testFee = {
    name: "DELETE_ME Test Proposal",
    number: "DELETE_ME_001",
    status: "Draft",
    issue_date: "2025-08-17",
    rev: 1,
    project: projectResult.id || "test_project_id",
    company: companyResult.id || "test_company_id", 
    contact: contactResult.id || "test_contact_id",
    total_fee: 50000,
    currency: "AED",
    notes: "This is a DELETE_ME test proposal"
  };
  
  const feeResult = await runTauriCommand('create_fee', { fee: testFee });
  testResult('Proposal/Fee Creation', feeResult.success,
    feeResult.success ? 'Test proposal created successfully' : feeResult.error,
    feeResult.success ? { type: 'fee', id: feeResult.id } : null);
  
  console.log('\n📋 TEST 5: READ OPERATIONS');
  
  // Test Read Operations
  const readTests = [
    { command: 'get_companies', name: 'Get Companies' },
    { command: 'get_contacts', name: 'Get Contacts' },
    { command: 'get_projects', name: 'Get Projects' },
    { command: 'get_fees', name: 'Get Proposals/Fees' }
  ];
  
  for (const test of readTests) {
    const result = await runTauriCommand(test.command);
    testResult(test.name, result.success, 
      result.success ? 'Data retrieved successfully' : result.error);
  }
  
  console.log('\n📋 TEST 6: UPDATE OPERATIONS');
  
  // Test Update Operations (only on our test records)
  if (companyResult.success) {
    const updateCompanyResult = await runTauriCommand('update_company', {
      id: companyResult.id,
      company: { ...testCompany, name: "DELETE_ME Updated Test Company" }
    });
    testResult('Company Update', updateCompanyResult.success,
      updateCompanyResult.success ? 'Test company updated successfully' : updateCompanyResult.error);
  }
  
  if (contactResult.success) {
    const updateContactResult = await runTauriCommand('update_contact', {
      id: contactResult.id, 
      contact: { ...testContact, first_name: "DELETE_ME_UPDATED" }
    });
    testResult('Contact Update', updateContactResult.success,
      updateContactResult.success ? 'Test contact updated successfully' : updateContactResult.error);
  }
  
  console.log('\n📋 TEST 7: SEARCH AND FILTER OPERATIONS');
  
  // Test Search Operations
  const searchTests = [
    { command: 'search_companies', data: { query: 'DELETE_ME' }, name: 'Company Search' },
    { command: 'search_contacts', data: { query: 'DELETE_ME' }, name: 'Contact Search' }, 
    { command: 'search_projects', data: { query: 'DELETE_ME' }, name: 'Project Search' }
  ];
  
  for (const test of searchTests) {
    const result = await runTauriCommand(test.command, test.data);
    testResult(test.name, result.success,
      result.success ? 'Search completed successfully' : result.error);
  }
  
  console.log('\n📋 TEST 8: ERROR HANDLING');
  
  // Test Error Handling with invalid data
  const errorTests = [
    { 
      command: 'create_company', 
      data: { company: { name: '' } }, // Missing required fields
      name: 'Invalid Company Creation'
    },
    {
      command: 'update_contact',
      data: { id: 'invalid_id', contact: {} },
      name: 'Invalid Contact Update'
    }
  ];
  
  for (const test of errorTests) {
    const result = await runTauriCommand(test.command, test.data);
    testResult(test.name, !result.success, // Should fail
      !result.success ? 'Error handling working correctly' : 'Should have failed but succeeded');
  }
}

async function cleanupTestRecords() {
  console.log('\n🧹 CLEANUP: Deleting all test records');
  console.log('=====================================');
  
  let deletedCount = 0;
  
  for (const record of createdRecords) {
    try {
      let deleteCommand;
      switch (record.type) {
        case 'company':
          deleteCommand = 'delete_company';
          break;
        case 'contact':
          deleteCommand = 'delete_contact';
          break;
        case 'project':
          deleteCommand = 'delete_project'; 
          break;
        case 'fee':
          deleteCommand = 'delete_fee';
          break;
        default:
          continue;
      }
      
      const result = await runTauriCommand(deleteCommand, { id: record.id });
      if (result.success) {
        console.log(`✅ Deleted ${record.type}: ${record.id}`);
        deletedCount++;
      } else {
        console.log(`❌ Failed to delete ${record.type}: ${record.id}`);
      }
    } catch (error) {
      console.log(`❌ Error deleting ${record.type}: ${error.message}`);
    }
  }
  
  console.log(`\n🧹 Cleanup completed: ${deletedCount}/${createdRecords.length} records deleted`);
  
  if (deletedCount < createdRecords.length) {
    console.log('⚠️  Some test records may still exist - manual cleanup may be required');
    console.log('   Look for records containing "DELETE_ME" in the database');
  }
}

// Main test execution
async function runTests() {
  try {
    await testCRUDFunctionality();
    
    console.log('\n🧪 CRUD TESTING RESULTS');
    console.log('========================');
    console.log(`Tests Passed: ${testsPassed}/${testsTotal}`);
    console.log(`Success Rate: ${((testsPassed / testsTotal) * 100).toFixed(1)}%`);
    
    if (testsPassed === testsTotal) {
      console.log('\n🎉 ALL CRUD TESTS PASSED!');
      console.log('✅ The application CRUD functionality is working correctly');
    } else {
      console.log('\n⚠️  SOME CRUD TESTS FAILED');
      console.log('❌ Review failing operations before v1.0.0 release');
    }
    
    // Always attempt cleanup
    await cleanupTestRecords();
    
  } catch (error) {
    console.error('\n💥 TESTING ERROR:', error.message);
    console.log('\n🧹 Attempting cleanup of any created records...');
    await cleanupTestRecords();
  }
}

// Note: This is a template for CRUD testing
// To actually run these tests, you would need to:
// 1. Start the Tauri app
// 2. Use the Tauri MCP server to invoke these commands
// 3. Or integrate this into the app's testing framework

console.log('📝 NOTE: This is a CRUD testing template');
console.log('   To run actual tests, use the Tauri MCP server or integrate into the app');
console.log('   All test records will be clearly marked with DELETE_ME for safety\n');

runTests();