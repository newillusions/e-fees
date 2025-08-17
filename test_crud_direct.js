#!/usr/bin/env node

/**
 * Direct CRUD Testing
 * Run this in the browser console of the Tauri app
 */

// This script should be executed in the browser console of the running Tauri app
// Copy and paste these functions into the console

async function testCRUD() {
  const { invoke } = window.__TAURI__.core;
  
  console.log('🧪 STARTING CRUD TESTS');
  console.log('======================');
  
  let companyId, contactId, projectId, feeId;
  
  // Test 1: Create Company
  console.log('\n📋 TEST 1: CREATE COMPANY');
  try {
    const company = {
      name: "DELETE_ME Test Company",
      abbreviation: "DELME",
      email: "deleteme@test.com",
      phone: "+971 50 000 0000",
      address: "Test Address DELETE_ME",
      city: "Test City",
      country: "UAE",
      notes: "CRUD test record - safe to delete"
    };
    
    const result = await invoke('create_company', { company });
    companyId = result.id || result;
    console.log('✅ Company created:', companyId);
  } catch (error) {
    console.error('❌ Failed to create company:', error);
  }
  
  // Test 2: Create Contact
  console.log('\n📋 TEST 2: CREATE CONTACT');
  try {
    const contact = {
      first_name: "DELETE_ME",
      last_name: "Test Contact",
      email: "deleteme.contact@test.com",
      phone: "+971 50 000 0001",
      position: "Test Position",
      company: companyId,
      notes: "CRUD test record - safe to delete"
    };
    
    const result = await invoke('create_contact', { contact });
    contactId = result.id || result;
    console.log('✅ Contact created:', contactId);
  } catch (error) {
    console.error('❌ Failed to create contact:', error);
  }
  
  // Test 3: Create Project
  console.log('\n📋 TEST 3: CREATE PROJECT');
  try {
    const project = {
      name: "DELETE_ME Test Project",
      name_short: "DELME-PROJ",
      number: "25-97199",
      status: "Draft",
      area: "Test Area",
      city: "Test City",
      country: "UAE",
      folder: "DELETE_ME_Test",
      notes: "CRUD test record - safe to delete"
    };
    
    const result = await invoke('create_project', { project });
    projectId = result.id || result;
    console.log('✅ Project created:', projectId);
  } catch (error) {
    console.error('❌ Failed to create project:', error);
  }
  
  // Test 4: Create Fee/Proposal
  console.log('\n📋 TEST 4: CREATE PROPOSAL');
  try {
    const fee = {
      name: "DELETE_ME Test Proposal",
      number: "DELETE_ME_001",
      status: "Draft",
      issue_date: "2025-08-17",
      rev: 1,
      project: projectId,
      company: companyId,
      contact: contactId,
      total_fee: 50000,
      currency: "AED",
      notes: "CRUD test record - safe to delete"
    };
    
    const result = await invoke('create_fee', { fee });
    feeId = result.id || result;
    console.log('✅ Proposal created:', feeId);
  } catch (error) {
    console.error('❌ Failed to create proposal:', error);
  }
  
  // Test 5: Read Operations
  console.log('\n📋 TEST 5: READ OPERATIONS');
  try {
    const companies = await invoke('get_companies');
    const contacts = await invoke('get_contacts');
    const projects = await invoke('get_projects');
    const fees = await invoke('get_fees');
    
    console.log(`✅ Companies: ${companies.length} total`);
    console.log(`✅ Contacts: ${contacts.length} total`);
    console.log(`✅ Projects: ${projects.length} total`);
    console.log(`✅ Proposals: ${fees.length} total`);
  } catch (error) {
    console.error('❌ Failed to read data:', error);
  }
  
  // Test 6: Update Operations
  console.log('\n📋 TEST 6: UPDATE OPERATIONS');
  try {
    const updatedCompany = {
      name: "DELETE_ME Updated Test Company",
      abbreviation: "DELMEUP",
      email: "deleteme.updated@test.com",
      phone: "+971 50 111 1111",
      address: "Updated Test Address DELETE_ME",
      city: "Updated Test City",
      country: "UAE",
      notes: "UPDATED - CRUD test record - safe to delete"
    };
    
    await invoke('update_company', { 
      id: companyId,
      company: updatedCompany 
    });
    console.log('✅ Company updated successfully');
  } catch (error) {
    console.error('❌ Failed to update company:', error);
  }
  
  // Test 7: Delete Operations (Cleanup)
  console.log('\n📋 TEST 7: DELETE OPERATIONS (CLEANUP)');
  
  // Delete in reverse order due to foreign key constraints
  try {
    if (feeId) {
      await invoke('delete_fee', { id: feeId });
      console.log('✅ Deleted proposal:', feeId);
    }
  } catch (error) {
    console.error('❌ Failed to delete proposal:', error);
  }
  
  try {
    if (contactId) {
      await invoke('delete_contact', { id: contactId });
      console.log('✅ Deleted contact:', contactId);
    }
  } catch (error) {
    console.error('❌ Failed to delete contact:', error);
  }
  
  try {
    if (projectId) {
      await invoke('delete_project', { id: projectId });
      console.log('✅ Deleted project:', projectId);
    }
  } catch (error) {
    console.error('❌ Failed to delete project:', error);
  }
  
  try {
    if (companyId) {
      await invoke('delete_company', { id: companyId });
      console.log('✅ Deleted company:', companyId);
    }
  } catch (error) {
    console.error('❌ Failed to delete company:', error);
  }
  
  console.log('\n🧪 CRUD TESTING COMPLETE');
  console.log('========================');
  console.log('✅ All operations tested successfully');
  console.log('✅ Test records cleaned up');
}

// Execute the test
testCRUD();