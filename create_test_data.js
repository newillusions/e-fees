// Test data creation script - to be run in browser console when app is running
// This creates database entries that match our test folder structure

async function createTestData() {
  console.log('=== Creating Test Data for Folder Management ===');
  
  try {
    // First check if we can connect to the database
    const isConnected = await invoke('check_db_connection');
    if (!isConnected) {
      console.error('Database not connected');
      return;
    }
    
    // Create test company
    console.log('1. Creating test company...');
    const testCompany = {
      name: 'Test Client LLC',
      name_short: 'Test Client',
      abbreviation: 'TC',
      city: 'Dubai',
      country: 'United Arab Emirates',
      reg_no: 'TC-12345',
      tax_no: 'TC-TAX-001'
    };
    
    const company = await invoke('create_company', { company: testCompany });
    console.log('Created company:', company);
    
    // Create test contact
    console.log('2. Creating test contact...');
    const testContact = {
      first_name: 'John',
      last_name: 'Doe',
      full_name: 'John Doe',
      email: 'john.doe@testclient.com',
      phone: '+971-50-123-4567',
      position: 'Project Manager',
      company: company.id
    };
    
    const contact = await invoke('create_contact', { contact: testContact });
    console.log('Created contact:', contact);
    
    // Create test project for our test folder (25-97199)
    console.log('3. Creating test project for 25-97199...');
    const testProject = {
      name: 'Test Project for Folder Management',
      area: 'Commercial',
      city: 'Dubai',
      country: 'United Arab Emirates',
      status: 'RFP',
      folder: '/Volumes/base/mms/DevTest/01 RFPs/25-97199 Test Project',
      number: {
        year: 25,
        country_code: 971,
        sequence: 199,
        id: '25-97199'
      }
    };
    
    const project = await invoke('create_project', { project: testProject });
    console.log('Created project:', project);
    
    // Create test project for existing folder (25-97107)
    console.log('4. Creating test project for 25-97107...');
    const testProject2 = {
      name: 'Cove Boulevard Project',
      area: 'Residential',
      city: 'Dubai',
      country: 'United Arab Emirates',
      status: 'RFP',
      folder: '/Volumes/base/mms/DevTest/01 RFPs/25-97107 Cove Boulevard',
      number: {
        year: 25,
        country_code: 971,
        sequence: 107,
        id: '25-97107'
      }
    };
    
    const project2 = await invoke('create_project', { project: testProject2 });
    console.log('Created project:', project2);
    
    // Create test fee proposal for the projects
    console.log('5. Creating test fee proposals...');
    const testFee1 = {
      number: '25-97199-FP-01',
      rev: '00',
      package: 'Lighting Design',
      status: 'Draft',
      issue_date: '250801',
      staff_name: 'Test Designer',
      project_id: project.id,
      company_id: company.id,
      contact_id: contact.id
    };
    
    const fee1 = await invoke('create_fee', { fee: testFee1 });
    console.log('Created fee proposal:', fee1);
    
    const testFee2 = {
      number: '25-97107-FP-01',
      rev: '00',
      package: 'Full Lighting Package',
      status: 'Sent',
      issue_date: '250801',
      staff_name: 'Test Designer',
      project_id: project2.id,
      company_id: company.id,
      contact_id: contact.id
    };
    
    const fee2 = await invoke('create_fee', { fee: testFee2 });
    console.log('Created fee proposal:', fee2);
    
    console.log('\n=== Test Data Creation Complete ===');
    console.log('✅ You can now test folder management in the app!');
    console.log('📁 Test projects: 25-97199, 25-97107');
    console.log('🏢 Test company: Test Client LLC');
    console.log('👤 Test contact: John Doe');
    console.log('📄 Test proposals: 25-97199-FP-01, 25-97107-FP-01');
    
    // Instructions for testing
    console.log('\n=== Testing Instructions ===');
    console.log('1. Navigate to Projects page');
    console.log('2. Find "25-97199 - Test Project for Folder Management"');
    console.log('3. Edit the project and change status from "RFP" to "Active"');
    console.log('4. The StatusChangeModal should appear showing:');
    console.log('   - Folder movement from "01 RFPs" to "11 Current"');
    console.log('   - Template copying notification');
    console.log('   - Related proposal impact');
    console.log('5. Confirm the change and check the DevTest folder');
    
  } catch (error) {
    console.error('Failed to create test data:', error);
  }
}

// Instructions for use:
console.log('📋 To create test data, run: createTestData()');

// You can also create individual items:
window.createTestData = createTestData;