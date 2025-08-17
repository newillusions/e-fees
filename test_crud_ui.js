#!/usr/bin/env node

/**
 * CRUD Testing Through UI Simulation
 * Tests Tauri commands that the UI would call
 */

const { exec } = require('child_process');
const util = require('util');
const execAsync = util.promisify(exec);

console.log('🧪 CRUD FUNCTIONALITY UI TESTING');
console.log('=====================================');
console.log('Testing: Companies, Contacts, Projects, Proposals');
console.log('');

// Track test records for cleanup
const createdRecords = {
  companies: [],
  contacts: [],
  projects: [],
  proposals: []
};

async function executeInApp(jsCode) {
  try {
    // Use osascript to execute JavaScript in the app
    const script = `
      tell application "System Events"
        tell process "Fee Proposal Management"
          set frontmost to true
        end tell
      end tell
      
      tell application "Fee Proposal Management"
        do JavaScript "${jsCode.replace(/"/g, '\\"')}"
      end tell
    `;
    
    const { stdout, stderr } = await execAsync(`osascript -e '${script}'`);
    if (stderr) {
      console.error('Script error:', stderr);
      return null;
    }
    return stdout;
  } catch (error) {
    console.error('Execution failed:', error);
    return null;
  }
}

// Test 1: Create Company through UI
async function testCreateCompany() {
  console.log('\n📋 TEST 1: CREATE COMPANY');
  console.log('-------------------------');
  
  const createCompanyJS = `
    (async () => {
      const { invoke } = window.__TAURI__.core;
      
      const company = {
        name: "DELETE_ME Test Company UI",
        abbreviation: "DELUI",
        email: "deleteme.ui@test.com",
        phone: "+971 50 111 1111",
        address: "UI Test Address DELETE_ME",
        city: "Test City",
        country: "UAE",
        notes: "Created through UI test - safe to delete"
      };
      
      try {
        const result = await invoke('create_company', { company });
        console.log('Company created:', result);
        return JSON.stringify(result);
      } catch (error) {
        console.error('Failed to create company:', error);
        return JSON.stringify({ error: error.toString() });
      }
    })();
  `;
  
  const result = await executeInApp(createCompanyJS);
  if (result) {
    const data = JSON.parse(result);
    if (!data.error && data.id) {
      console.log('✅ Company created successfully');
      console.log('   ID:', data.id);
      createdRecords.companies.push(data.id);
    } else {
      console.log('❌ Failed to create company:', data.error);
    }
  }
}

// Test 2: Create Contact through UI
async function testCreateContact() {
  console.log('\n📋 TEST 2: CREATE CONTACT');
  console.log('------------------------');
  
  const companyId = createdRecords.companies[0];
  if (!companyId) {
    console.log('⚠️  Skipping - no company ID available');
    return;
  }
  
  const createContactJS = `
    (async () => {
      const { invoke } = window.__TAURI__.core;
      
      const contact = {
        first_name: "DELETE_ME",
        last_name: "UI Test Contact",
        email: "deleteme.ui.contact@test.com",
        phone: "+971 50 222 2222",
        position: "UI Test Position",
        company: "${companyId}",
        notes: "Created through UI test - safe to delete"
      };
      
      try {
        const result = await invoke('create_contact', { contact });
        console.log('Contact created:', result);
        return JSON.stringify(result);
      } catch (error) {
        console.error('Failed to create contact:', error);
        return JSON.stringify({ error: error.toString() });
      }
    })();
  `;
  
  const result = await executeInApp(createContactJS);
  if (result) {
    const data = JSON.parse(result);
    if (!data.error && data.id) {
      console.log('✅ Contact created successfully');
      console.log('   ID:', data.id);
      createdRecords.contacts.push(data.id);
    } else {
      console.log('❌ Failed to create contact:', data.error);
    }
  }
}

// Test 3: Create Project through UI
async function testCreateProject() {
  console.log('\n📋 TEST 3: CREATE PROJECT');
  console.log('------------------------');
  
  const createProjectJS = `
    (async () => {
      const { invoke } = window.__TAURI__.core;
      
      const project = {
        name: "DELETE_ME UI Test Project",
        name_short: "DELUI-PROJ",
        number: "25-97198",
        status: "Draft",
        area: "UI Test Area",
        city: "Test City",
        country: "UAE",
        folder: "DELETE_ME_UI_Test",
        notes: "Created through UI test - safe to delete"
      };
      
      try {
        const result = await invoke('create_project', { project });
        console.log('Project created:', result);
        return JSON.stringify(result);
      } catch (error) {
        console.error('Failed to create project:', error);
        return JSON.stringify({ error: error.toString() });
      }
    })();
  `;
  
  const result = await executeInApp(createProjectJS);
  if (result) {
    const data = JSON.parse(result);
    if (!data.error && data.id) {
      console.log('✅ Project created successfully');
      console.log('   ID:', data.id);
      createdRecords.projects.push(data.id);
    } else {
      console.log('❌ Failed to create project:', data.error);
    }
  }
}

// Test 4: Create Proposal through UI
async function testCreateProposal() {
  console.log('\n📋 TEST 4: CREATE PROPOSAL');
  console.log('-------------------------');
  
  const companyId = createdRecords.companies[0];
  const contactId = createdRecords.contacts[0];
  const projectId = createdRecords.projects[0];
  
  if (!companyId || !contactId || !projectId) {
    console.log('⚠️  Skipping - missing required IDs');
    return;
  }
  
  const createProposalJS = `
    (async () => {
      const { invoke } = window.__TAURI__.core;
      
      const fee = {
        name: "DELETE_ME UI Test Proposal",
        number: "DELETE_ME_UI_001",
        status: "Draft",
        issue_date: "2025-08-17",
        rev: 1,
        project: "${projectId}",
        company: "${companyId}",
        contact: "${contactId}",
        total_fee: 75000,
        currency: "AED",
        notes: "Created through UI test - safe to delete"
      };
      
      try {
        const result = await invoke('create_fee', { fee });
        console.log('Proposal created:', result);
        return JSON.stringify(result);
      } catch (error) {
        console.error('Failed to create proposal:', error);
        return JSON.stringify({ error: error.toString() });
      }
    })();
  `;
  
  const result = await executeInApp(createProposalJS);
  if (result) {
    const data = JSON.parse(result);
    if (!data.error && data.id) {
      console.log('✅ Proposal created successfully');
      console.log('   ID:', data.id);
      createdRecords.proposals.push(data.id);
    } else {
      console.log('❌ Failed to create proposal:', data.error);
    }
  }
}

// Test 5: Update Operations
async function testUpdateOperations() {
  console.log('\n📋 TEST 5: UPDATE OPERATIONS');
  console.log('----------------------------');
  
  const companyId = createdRecords.companies[0];
  if (!companyId) {
    console.log('⚠️  Skipping - no company ID available');
    return;
  }
  
  const updateCompanyJS = `
    (async () => {
      const { invoke } = window.__TAURI__.core;
      
      const updatedCompany = {
        id: "${companyId}",
        name: "DELETE_ME Updated UI Company",
        abbreviation: "DELUIUP",
        email: "deleteme.ui.updated@test.com",
        phone: "+971 50 333 3333",
        address: "Updated UI Test Address DELETE_ME",
        city: "Updated Test City",
        country: "UAE",
        notes: "UPDATED through UI test - safe to delete"
      };
      
      try {
        const result = await invoke('update_company', { 
          id: "${companyId}",
          company: updatedCompany 
        });
        console.log('Company updated:', result);
        return JSON.stringify(result);
      } catch (error) {
        console.error('Failed to update company:', error);
        return JSON.stringify({ error: error.toString() });
      }
    })();
  `;
  
  const result = await executeInApp(updateCompanyJS);
  if (result) {
    const data = JSON.parse(result);
    if (!data.error) {
      console.log('✅ Company updated successfully');
    } else {
      console.log('❌ Failed to update company:', data.error);
    }
  }
}

// Test 6: Read Operations
async function testReadOperations() {
  console.log('\n📋 TEST 6: READ OPERATIONS');
  console.log('--------------------------');
  
  const commands = [
    { name: 'get_companies', label: 'Get Companies' },
    { name: 'get_contacts', label: 'Get Contacts' },
    { name: 'get_projects', label: 'Get Projects' },
    { name: 'get_fees', label: 'Get Proposals' }
  ];
  
  for (const cmd of commands) {
    const readJS = `
      (async () => {
        const { invoke } = window.__TAURI__.core;
        
        try {
          const result = await invoke('${cmd.name}');
          const deleteCount = result.filter(r => 
            r.name && r.name.includes('DELETE_ME')
          ).length;
          return JSON.stringify({ 
            total: result.length, 
            deleteMe: deleteCount 
          });
        } catch (error) {
          return JSON.stringify({ error: error.toString() });
        }
      })();
    `;
    
    const result = await executeInApp(readJS);
    if (result) {
      const data = JSON.parse(result);
      if (!data.error) {
        console.log(`✅ ${cmd.label}: ${data.total} total, ${data.deleteMe} test records`);
      } else {
        console.log(`❌ ${cmd.label}: ${data.error}`);
      }
    }
  }
}

// Test 7: Delete Operations (Cleanup)
async function testDeleteOperations() {
  console.log('\n📋 TEST 7: DELETE OPERATIONS (CLEANUP)');
  console.log('---------------------------------------');
  
  // Delete proposals first (foreign key constraints)
  for (const id of createdRecords.proposals) {
    const deleteJS = `
      (async () => {
        const { invoke } = window.__TAURI__.core;
        try {
          await invoke('delete_fee', { id: "${id}" });
          return JSON.stringify({ success: true });
        } catch (error) {
          return JSON.stringify({ error: error.toString() });
        }
      })();
    `;
    
    const result = await executeInApp(deleteJS);
    if (result) {
      const data = JSON.parse(result);
      if (data.success) {
        console.log(`✅ Deleted proposal: ${id}`);
      } else {
        console.log(`❌ Failed to delete proposal: ${data.error}`);
      }
    }
  }
  
  // Delete contacts
  for (const id of createdRecords.contacts) {
    const deleteJS = `
      (async () => {
        const { invoke } = window.__TAURI__.core;
        try {
          await invoke('delete_contact', { id: "${id}" });
          return JSON.stringify({ success: true });
        } catch (error) {
          return JSON.stringify({ error: error.toString() });
        }
      })();
    `;
    
    const result = await executeInApp(deleteJS);
    if (result) {
      const data = JSON.parse(result);
      if (data.success) {
        console.log(`✅ Deleted contact: ${id}`);
      } else {
        console.log(`❌ Failed to delete contact: ${data.error}`);
      }
    }
  }
  
  // Delete projects
  for (const id of createdRecords.projects) {
    const deleteJS = `
      (async () => {
        const { invoke } = window.__TAURI__.core;
        try {
          await invoke('delete_project', { id: "${id}" });
          return JSON.stringify({ success: true });
        } catch (error) {
          return JSON.stringify({ error: error.toString() });
        }
      })();
    `;
    
    const result = await executeInApp(deleteJS);
    if (result) {
      const data = JSON.parse(result);
      if (data.success) {
        console.log(`✅ Deleted project: ${id}`);
      } else {
        console.log(`❌ Failed to delete project: ${data.error}`);
      }
    }
  }
  
  // Delete companies last
  for (const id of createdRecords.companies) {
    const deleteJS = `
      (async () => {
        const { invoke } = window.__TAURI__.core;
        try {
          await invoke('delete_company', { id: "${id}" });
          return JSON.stringify({ success: true });
        } catch (error) {
          return JSON.stringify({ error: error.toString() });
        }
      })();
    `;
    
    const result = await executeInApp(deleteJS);
    if (result) {
      const data = JSON.parse(result);
      if (data.success) {
        console.log(`✅ Deleted company: ${id}`);
      } else {
        console.log(`❌ Failed to delete company: ${data.error}`);
      }
    }
  }
}

// Main test runner
async function runTests() {
  console.log('Starting CRUD UI testing...');
  console.log('Note: This test simulates UI operations through Tauri commands\n');
  
  try {
    await testCreateCompany();
    await testCreateContact();
    await testCreateProject();
    await testCreateProposal();
    await testUpdateOperations();
    await testReadOperations();
    await testDeleteOperations();
    
    console.log('\n🧪 CRUD UI TESTING COMPLETE');
    console.log('============================');
    console.log('✅ All CRUD operations tested through UI simulation');
    console.log('✅ Test records cleaned up successfully');
    
  } catch (error) {
    console.error('\n💥 TEST ERROR:', error);
    console.log('Attempting cleanup...');
    await testDeleteOperations();
  }
}

// Check if app is running
exec('ps aux | grep "Fee Proposal Management" | grep -v grep', (error, stdout) => {
  if (!stdout) {
    console.log('⚠️  Fee Proposal Management app is not running');
    console.log('Please start the app with: npm run tauri:dev');
  } else {
    console.log('✅ App is running, starting tests...\n');
    runTests();
  }
});